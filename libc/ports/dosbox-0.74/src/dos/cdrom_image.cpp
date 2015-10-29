/*
 *  Copyright (C) 2002-2010  The DOSBox Team
 *
 *  This program is free software; you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation; either version 2 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program; if not, write to the Free Software
 *  Foundation, Inc., 59 Temple Place - Suite 330, Boston, MA 02111-1307, USA.
 */

/* $Id: cdrom_image.cpp,v 1.24 2009-03-19 20:45:42 c2woody Exp $ */

#include <cctype>
#include <cmath>
#include <cstdio>
#include <fstream>
#include <iostream>
#include <limits>
#include <limits.h> //GCC 2.95
#include <sstream>
#include <vector>
#include <sys/stat.h>
#include "cdrom.h"
#include "drives.h"
#include "support.h"
#include "setup.h"

#if !defined(WIN32)
#include <libgen.h>
#else
#include <string.h>
#endif

using namespace std;

#define MAX_LINE_LENGTH 512
#define MAX_FILENAME_LENGTH 256

CDROM_Interface_Image::BinaryFile::BinaryFile(const char *filename, bool &error)
{
	file = new ifstream(filename, ios::in | ios::binary);
	error = (file == NULL) || (file->fail());
}

CDROM_Interface_Image::BinaryFile::~BinaryFile()
{
	delete file;
}

bool CDROM_Interface_Image::BinaryFile::read(Bit8u *buffer, int seek, int count)
{
	file->seekg(seek, ios::beg);
	file->read((char*)buffer, count);
	return !(file->fail());
}

int CDROM_Interface_Image::BinaryFile::getLength()
{
	file->seekg(0, ios::end);
	int length = (int)file->tellg();
	if (file->fail()) return -1;
	return length;
}

#if defined(C_SDL_SOUND)
CDROM_Interface_Image::AudioFile::AudioFile(const char *filename, bool &error)
{
	Sound_AudioInfo desired = {AUDIO_S16, 2, 44100};
	sample = Sound_NewSampleFromFile(filename, &desired, RAW_SECTOR_SIZE);
	lastCount = RAW_SECTOR_SIZE;
	lastSeek = 0;
	error = (sample == NULL);
}

CDROM_Interface_Image::AudioFile::~AudioFile()
{
	Sound_FreeSample(sample);
}

bool CDROM_Interface_Image::AudioFile::read(Bit8u *buffer, int seek, int count)
{
	if (lastCount != count) {
		int success = Sound_SetBufferSize(sample, count);
		if (!success) return false;
	}
	if (lastSeek != (seek - count)) {
		int success = Sound_Seek(sample, (int)((double)(seek) / 176.4f));
		if (!success) return false;
	}
	lastSeek = seek;
	int bytes = Sound_Decode(sample);
	if (bytes < count) {
		memcpy(buffer, sample->buffer, bytes);
		memset(buffer + bytes, 0, count - bytes);
	} else {
		memcpy(buffer, sample->buffer, count);
	}

	return !(sample->flags & SOUND_SAMPLEFLAG_ERROR);
}

int CDROM_Interface_Image::AudioFile::getLength()
{
	int time = 1;
	int shift = 0;
	if (!(sample->flags & SOUND_SAMPLEFLAG_CANSEEK)) return -1;

	while (true) {
		int success = Sound_Seek(sample, (unsigned int)(shift + time));
		if (!success) {
			if (time == 1) return lround((double)shift * 176.4f);
			shift += time >> 1;
			time = 1;
		} else {
			if (time > ((numeric_limits<int>::max() - shift) / 2)) return -1;
			time = time << 1;
		}
	}
}
#endif

// initialize static members
int CDROM_Interface_Image::refCount = 0;
CDROM_Interface_Image* CDROM_Interface_Image::images[26];
CDROM_Interface_Image::imagePlayer CDROM_Interface_Image::player = {
	NULL, NULL, NULL, {0}, 0, 0, 0, false, false };


CDROM_Interface_Image::CDROM_Interface_Image(Bit8u subUnit)
{
	images[subUnit] = this;
	if (refCount == 0) {
		player.mutex = SDL_CreateMutex();
		if (!player.channel) {
			player.channel = MIXER_AddChannel(&CDAudioCallBack, 44100, "CDAUDIO");
		}
		player.channel->Enable(true);
	}
	refCount++;
}

CDROM_Interface_Image::~CDROM_Interface_Image()
{
	refCount--;
	if (player.cd == this) player.cd = NULL;
	ClearTracks();
	if (refCount == 0) {
		SDL_DestroyMutex(player.mutex);
		player.channel->Enable(false);
	}
}

void CDROM_Interface_Image::InitNewMedia()
{
}

bool CDROM_Interface_Image::SetDevice(char* path, int forceCD)
{
	if (LoadCueSheet(path)) return true;
	if (LoadIsoFile(path)) return true;

	// print error message on dosbox console
	char buf[MAX_LINE_LENGTH];
	snprintf(buf, MAX_LINE_LENGTH, "Could not load image file: %s\n", path);
	Bit16u size = (Bit16u)strlen(buf);
	DOS_WriteFile(STDOUT, (Bit8u*)buf, &size);
	return false;
}

bool CDROM_Interface_Image::GetUPC(unsigned char& attr, char* upc)
{
	attr = 0;
	strcpy(upc, this->mcn.c_str());
	return true;
}

bool CDROM_Interface_Image::GetAudioTracks(int& stTrack, int& end, TMSF& leadOut)
{
	stTrack = 1;
	end = (int)(tracks.size() - 1);
	FRAMES_TO_MSF(tracks[tracks.size() - 1].start + 150, &leadOut.min, &leadOut.sec, &leadOut.fr);
	return true;
}

bool CDROM_Interface_Image::GetAudioTrackInfo(int track, TMSF& start, unsigned char& attr)
{
	if (track < 1 || track > (int)tracks.size()) return false;
	FRAMES_TO_MSF(tracks[track - 1].start + 150, &start.min, &start.sec, &start.fr);
	attr = tracks[track - 1].attr;
	return true;
}

bool CDROM_Interface_Image::GetAudioSub(unsigned char& attr, unsigned char& track, unsigned char& index, TMSF& relPos, TMSF& absPos)
{
	int cur_track = GetTrack(player.currFrame);
	if (cur_track < 1) return false;
	track = (unsigned char)cur_track;
	attr = tracks[track - 1].attr;
	index = 1;
	FRAMES_TO_MSF(player.currFrame + 150, &absPos.min, &absPos.sec, &absPos.fr);
	FRAMES_TO_MSF(player.currFrame - tracks[track - 1].start + 150, &relPos.min, &relPos.sec, &relPos.fr);
	return true;
}

bool CDROM_Interface_Image::GetAudioStatus(bool& playing, bool& pause)
{
	playing = player.isPlaying;
	pause = player.isPaused;
	return true;
}

bool CDROM_Interface_Image::GetMediaTrayStatus(bool& mediaPresent, bool& mediaChanged, bool& trayOpen)
{
	mediaPresent = true;
	mediaChanged = false;
	trayOpen = false;
	return true;
}

bool CDROM_Interface_Image::PlayAudioSector(unsigned long start,unsigned long len)
{
	// We might want to do some more checks. E.g valid start and length
	SDL_mutexP(player.mutex);
	player.cd = this;
	player.currFrame = start;
	player.targetFrame = start + len;
	int track = GetTrack(start) - 1;
	if(track >= 0 && tracks[track].attr == 0x40) {
		LOG(LOG_MISC,LOG_WARN)("Game tries to play the data track. Not doing this");
		player.isPlaying = false;
		//Unclear wether return false should be here.
		//specs say that this function returns at once and games should check the status wether the audio is actually playing
		//Real drives either fail or succeed as well
	} else player.isPlaying = true;
	player.isPaused = false;
	SDL_mutexV(player.mutex);
	return true;
}

bool CDROM_Interface_Image::PauseAudio(bool resume)
{
	if (!player.isPlaying) return false;
	player.isPaused = !resume;
	return true;
}

bool CDROM_Interface_Image::StopAudio(void)
{
	player.isPlaying = false;
	player.isPaused = false;
	return true;
}

bool CDROM_Interface_Image::ReadSectors(PhysPt buffer, bool raw, unsigned long sector, unsigned long num)
{
	int sectorSize = raw ? RAW_SECTOR_SIZE : COOKED_SECTOR_SIZE;
	Bitu buflen = num * sectorSize;
	Bit8u* buf = new Bit8u[buflen];

	bool success = true; //Gobliiins reads 0 sectors
	for(unsigned long i = 0; i < num; i++) {
		success = ReadSector(&buf[i * sectorSize], raw, sector + i);
		if (!success) break;
	}

	MEM_BlockWrite(buffer, buf, buflen);
	delete[] buf;

	return success;
}

bool CDROM_Interface_Image::LoadUnloadMedia(bool unload)
{
	return true;
}

int CDROM_Interface_Image::GetTrack(int sector)
{
	vector<Track>::iterator i = tracks.begin();
	vector<Track>::iterator end = tracks.end() - 1;

	while(i != end) {
		Track &curr = *i;
		Track &next = *(i + 1);
		if (curr.start <= sector && sector < next.start) return curr.number;
		i++;
	}
	return -1;
}

bool CDROM_Interface_Image::ReadSector(Bit8u *buffer, bool raw, unsigned long sector)
{
	int track = GetTrack(sector) - 1;
	if (track < 0) return false;

	int seek = tracks[track].skip + (sector - tracks[track].start) * tracks[track].sectorSize;
	int length = (raw ? RAW_SECTOR_SIZE : COOKED_SECTOR_SIZE);
	if (tracks[track].sectorSize != RAW_SECTOR_SIZE && raw) return false;
	if (tracks[track].sectorSize == RAW_SECTOR_SIZE && !tracks[track].mode2 && !raw) seek += 16;
	if (tracks[track].mode2 && !raw) seek += 24;

	return tracks[track].file->read(buffer, seek, length);
}

void CDROM_Interface_Image::CDAudioCallBack(Bitu len)
{
	len *= 4;       // 16 bit, stereo
	if (!len) return;
	if (!player.isPlaying || player.isPaused) {
		player.channel->AddSilence();
		return;
	}

	SDL_mutexP(player.mutex);
	while (player.bufLen < (Bits)len) {
		bool success;
		if (player.targetFrame > player.currFrame)
			success = player.cd->ReadSector(&player.buffer[player.bufLen], true, player.currFrame);
		else success = false;

		if (success) {
			player.currFrame++;
			player.bufLen += RAW_SECTOR_SIZE;
		} else {
			memset(&player.buffer[player.bufLen], 0, len - player.bufLen);
			player.bufLen = len;
			player.isPlaying = false;
		}
	}
	SDL_mutexV(player.mutex);
#if defined(WORDS_BIGENDIAN)
	player.channel->AddSamples_s16_nonnative(len/4,(Bit16s *)player.buffer);
#else
	player.channel->AddSamples_s16(len/4,(Bit16s *)player.buffer);
#endif
	memmove(player.buffer, &player.buffer[len], player.bufLen - len);
	player.bufLen -= len;
}

bool CDROM_Interface_Image::LoadIsoFile(char* filename)
{
	tracks.clear();

	// data track
	Track track = {0, 0, 0, 0, 0, 0, false, NULL};
	bool error;
	track.file = new BinaryFile(filename, error);
	if (error) {
		delete track.file;
		return false;
	}
	track.number = 1;
	track.attr = 0x40;//data

	// try to detect iso type
	if (CanReadPVD(track.file, COOKED_SECTOR_SIZE, false)) {
		track.sectorSize = COOKED_SECTOR_SIZE;
		track.mode2 = false;
	} else if (CanReadPVD(track.file, RAW_SECTOR_SIZE, false)) {
		track.sectorSize = RAW_SECTOR_SIZE;
		track.mode2 = false;
	} else if (CanReadPVD(track.file, 2336, true)) {
		track.sectorSize = 2336;
		track.mode2 = true;
	} else if (CanReadPVD(track.file, RAW_SECTOR_SIZE, true)) {
		track.sectorSize = RAW_SECTOR_SIZE;
		track.mode2 = true;
	} else return false;

	track.length = track.file->getLength() / track.sectorSize;
	tracks.push_back(track);

	// leadout track
	track.number = 2;
	track.attr = 0;
	track.start = track.length;
	track.length = 0;
	track.file = NULL;
	tracks.push_back(track);

	return true;
}

bool CDROM_Interface_Image::CanReadPVD(TrackFile *file, int sectorSize, bool mode2)
{
	Bit8u pvd[COOKED_SECTOR_SIZE];
	int seek = 16 * sectorSize;	// first vd is located at sector 16
	if (sectorSize == RAW_SECTOR_SIZE && !mode2) seek += 16;
	if (mode2) seek += 24;
	file->read(pvd, seek, COOKED_SECTOR_SIZE);
	// pvd[0] = descriptor type, pvd[1..5] = standard identifier, pvd[6] = iso version
	return (pvd[0] == 1 && !strncmp((char*)(&pvd[1]), "CD001", 5) && pvd[6] == 1);
}

#if defined(WIN32)
static string dirname(char * file) {
	char * sep = strrchr(file, '\\');
	if (sep == NULL)
		sep = strrchr(file, '/');
	if (sep == NULL)
		return "";
	else {
		int len = (int)(sep - file);
		char tmp[MAX_FILENAME_LENGTH];
		safe_strncpy(tmp, file, len+1);
		return tmp;
	}
}
#endif

bool CDROM_Interface_Image::LoadCueSheet(char *cuefile)
{
	return false;
	/*
	Track track = {0, 0, 0, 0, 0, 0, false, NULL};
	tracks.clear();
	int shift = 0;
	int currPregap = 0;
	int totalPregap = 0;
	int prestart = 0;
	bool success;
	bool canAddTrack = false;
	char tmp[MAX_FILENAME_LENGTH];	// dirname can change its argument
	safe_strncpy(tmp, cuefile, MAX_FILENAME_LENGTH);
	string pathname(dirname(tmp));
	ifstream in;
	in.open(cuefile, ios::in);
	if (in.fail()) return false;

	while(!in.eof()) {
		// get next line
		char buf[MAX_LINE_LENGTH];
		in.getline(buf, MAX_LINE_LENGTH);
		if (in.fail() && !in.eof()) return false;  // probably a binary file
		istringstream line(buf);

		string command;
		GetCueKeyword(command, line);

		if (command == "TRACK") {
			if (canAddTrack) success = AddTrack(track, shift, prestart, totalPregap, currPregap);
			else success = true;

			track.start = 0;
			track.skip = 0;
			currPregap = 0;
			prestart = 0;

			line >> track.number;
			string type;
			GetCueKeyword(type, line);

			if (type == "AUDIO") {
				track.sectorSize = RAW_SECTOR_SIZE;
				track.attr = 0;
				track.mode2 = false;
			} else if (type == "MODE1/2048") {
				track.sectorSize = COOKED_SECTOR_SIZE;
				track.attr = 0x40;
				track.mode2 = false;
			} else if (type == "MODE1/2352") {
				track.sectorSize = RAW_SECTOR_SIZE;
				track.attr = 0x40;
				track.mode2 = false;
			} else if (type == "MODE2/2336") {
				track.sectorSize = 2336;
				track.attr = 0x40;
				track.mode2 = true;
			} else if (type == "MODE2/2352") {
				track.sectorSize = RAW_SECTOR_SIZE;
				track.attr = 0x40;
				track.mode2 = true;
			} else success = false;

			canAddTrack = true;
		}
		else if (command == "INDEX") {
			int index;
			line >> index;
			int frame;
			success = GetCueFrame(frame, line);

			if (index == 1) track.start = frame;
			else if (index == 0) prestart = frame;
			// ignore other indices
		}
		else if (command == "FILE") {
			if (canAddTrack) success = AddTrack(track, shift, prestart, totalPregap, currPregap);
			else success = true;
			canAddTrack = false;

			string filename;
			GetCueString(filename, line);
			GetRealFileName(filename, pathname);
			string type;
			GetCueKeyword(type, line);

			track.file = NULL;
			bool error = true;
			if (type == "BINARY") {
				track.file = new BinaryFile(filename.c_str(), error);
			}
#if defined(C_SDL_SOUND)
			//The next if has been surpassed by the else, but leaving it in as not
			//to break existing cue sheets that depend on this.(mine with OGG tracks specifying MP3 as type)
			else if (type == "WAVE" || type == "AIFF" || type == "MP3") {
				track.file = new AudioFile(filename.c_str(), error);
			} else {
				const Sound_DecoderInfo **i;
				for (i = Sound_AvailableDecoders(); *i != NULL; i++) {
					if (*(*i)->extensions == type) {
						track.file = new AudioFile(filename.c_str(), error);
						break;
					}
				}
			}
#endif
			if (error) {
				delete track.file;
				success = false;
			}
		}
		else if (command == "PREGAP") success = GetCueFrame(currPregap, line);
		else if (command == "CATALOG") success = GetCueString(mcn, line);
		// ignored commands
		else if (command == "CDTEXTFILE" || command == "FLAGS" || command == "ISRC"
			|| command == "PERFORMER" || command == "POSTGAP" || command == "REM"
			|| command == "SONGWRITER" || command == "TITLE" || command == "") success = true;
		// failure
		else success = false;

		if (!success) return false;
	}
	// add last track
	if (!AddTrack(track, shift, prestart, totalPregap, currPregap)) return false;

	// add leadout track
	track.number++;
	track.attr = 0;//sync with load iso
	track.start = 0;
	track.length = 0;
	track.file = NULL;
	if(!AddTrack(track, shift, 0, totalPregap, 0)) return false;

	return true;
	*/
}

bool CDROM_Interface_Image::AddTrack(Track &curr, int &shift, int prestart, int &totalPregap, int currPregap)
{
	// frames between index 0(prestart) and 1(curr.start) must be skipped
	int skip;
	if (prestart > 0) {
		if (prestart > curr.start) return false;
		skip = curr.start - prestart;
	} else skip = 0;

	// first track (track number must be 1)
	if (tracks.empty()) {
		if (curr.number != 1) return false;
		curr.skip = skip * curr.sectorSize;
		curr.start += currPregap;
		totalPregap = currPregap;
		tracks.push_back(curr);
		return true;
	}

	Track &prev = *(tracks.end() - 1);

	// current track consumes data from the same file as the previous
	if (prev.file == curr.file) {
		curr.start += shift;
		prev.length = curr.start + totalPregap - prev.start - skip;
		curr.skip += prev.skip + prev.length * prev.sectorSize + skip * curr.sectorSize;
		totalPregap += currPregap;
		curr.start += totalPregap;
	// current track uses a different file as the previous track
	} else {
		int tmp = prev.file->getLength() - prev.skip;
		prev.length = tmp / prev.sectorSize;
		if (tmp % prev.sectorSize != 0) prev.length++; // padding

		curr.start += prev.start + prev.length + currPregap;
		curr.skip = skip * curr.sectorSize;
		shift += prev.start + prev.length;
		totalPregap = currPregap;
	}

	// error checks
	if (curr.number <= 1) return false;
	if (prev.number + 1 != curr.number) return false;
	if (curr.start < prev.start + prev.length) return false;
	if (curr.length < 0) return false;

	tracks.push_back(curr);
	return true;
}

bool CDROM_Interface_Image::HasDataTrack(void)
{
	//Data track has attribute 0x40
	for(track_it it = tracks.begin(); it != tracks.end(); it++) {
		if ((*it).attr == 0x40) return true;
	}
	return false;
}


bool CDROM_Interface_Image::GetRealFileName(string &filename, string &pathname)
{
	// check if file exists
	struct stat test;
	if (stat(filename.c_str(), &test) == 0) return true;

	// check if file with path relative to cue file exists
	string tmpstr(pathname + "/" + filename);
	if (stat(tmpstr.c_str(), &test) == 0) {
		filename = tmpstr;
		return true;
	}
	// finally check if file is in a dosbox local drive
	char fullname[CROSS_LEN];
	char tmp[CROSS_LEN];
	safe_strncpy(tmp, filename.c_str(), CROSS_LEN);
	Bit8u drive;
	if (!DOS_MakeName(tmp, fullname, &drive)) return false;

	localDrive *ldp = dynamic_cast<localDrive*>(Drives[drive]);
	if (ldp) {
		ldp->GetSystemFilename(tmp, fullname);
		if (stat(tmp, &test) == 0) {
			filename = tmp;
			return true;
		}
	}

	return false;
}

bool CDROM_Interface_Image::GetCueKeyword(string &keyword, istream &in)
{
	in >> keyword;
	for(Bitu i = 0; i < keyword.size(); i++) keyword[i] = toupper(keyword[i]);

	return true;
}

bool CDROM_Interface_Image::GetCueFrame(int &frames, istream &in)
{
	string msf;
	in >> msf;
	int min, sec, fr;
	bool success = sscanf(msf.c_str(), "%d:%d:%d", &min, &sec, &fr) == 3;
	frames = MSF_TO_FRAMES(min, sec, fr);

	return success;
}

bool CDROM_Interface_Image::GetCueString(string &str, istream &in)
{
	int pos = (int)in.tellg();
	in >> str;
	if (str[0] == '\"') {
		if (str[str.size() - 1] == '\"') {
			str.assign(str, 1, str.size() - 2);
		} else {
			in.seekg(pos, ios::beg);
			char buffer[MAX_FILENAME_LENGTH];
			in.getline(buffer, MAX_FILENAME_LENGTH, '\"');	// skip
			in.getline(buffer, MAX_FILENAME_LENGTH, '\"');
			str = buffer;
		}
	}
	return true;
}

void CDROM_Interface_Image::ClearTracks()
{
	vector<Track>::iterator i = tracks.begin();
	vector<Track>::iterator end = tracks.end();

	TrackFile* last = NULL;
	while(i != end) {
		Track &curr = *i;
		if (curr.file != last) {
			delete curr.file;
			last = curr.file;
		}
		i++;
	}
	tracks.clear();
}

void CDROM_Image_Destroy(Section*) {
#if defined(C_SDL_SOUND)
	Sound_Quit();
#endif
}

void CDROM_Image_Init(Section* section) {
#if defined(C_SDL_SOUND)
	Sound_Init();
	section->AddDestroyFunction(CDROM_Image_Destroy, false);
#endif
}
