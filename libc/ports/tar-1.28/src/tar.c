/* A tar (tape archiver) program.

   Copyright 1988, 1992-1997, 1999-2001, 2003-2007, 2012-2014 Free
   Software Foundation, Inc.

   Written by John Gilmore, starting 1985-08-25.

   This program is free software; you can redistribute it and/or modify it
   under the terms of the GNU General Public License as published by the
   Free Software Foundation; either version 3, or (at your option) any later
   version.

   This program is distributed in the hope that it will be useful, but
   WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General
   Public License for more details.

   You should have received a copy of the GNU General Public License along
   with this program.  If not, see <http://www.gnu.org/licenses/>.  */

#include <system.h>

#include <fnmatch.h>
#include <argp.h>
#include <argp-namefrob.h>
#include <argp-fmtstream.h>
#include <argp-version-etc.h>

#include <signal.h>
#if ! defined SIGCHLD && defined SIGCLD
# define SIGCHLD SIGCLD
#endif

/* The following causes "common.h" to produce definitions of all the global
   variables, rather than just "extern" declarations of them.  GNU tar does
   depend on the system loader to preset all GLOBAL variables to neutral (or
   zero) values; explicit initialization is usually not done.  */
#define GLOBAL
#include "common.h"

#include <argmatch.h>
#include <closeout.h>
#include <configmake.h>
#include <exitfail.h>
#include <parse-datetime.h>
#include <rmt.h>
#include <rmt-command.h>
#include <prepargs.h>
#include <quotearg.h>
#include <version-etc.h>
#include <xstrtol.h>
#include <stdopen.h>
#include <priv-set.h>
#include <savedir.h>

/* Local declarations.  */

#ifndef DEFAULT_ARCHIVE_FORMAT
# define DEFAULT_ARCHIVE_FORMAT GNU_FORMAT
#endif

#ifndef DEFAULT_ARCHIVE
# define DEFAULT_ARCHIVE "tar.out"
#endif

#ifndef DEFAULT_BLOCKING
# define DEFAULT_BLOCKING 20
#endif

/* Print a message if not all links are dumped */
static int check_links_option;

/* Number of allocated tape drive names.  */
static size_t allocated_archive_names;


/* Miscellaneous.  */

/* Name of option using stdin.  */
static const char *stdin_used_by;

/* Doesn't return if stdin already requested.  */
void
request_stdin (const char *option)
{
  if (stdin_used_by)
    USAGE_ERROR ((0, 0, _("Options '%s' and '%s' both want standard input"),
		  stdin_used_by, option));

  stdin_used_by = option;
}

extern int rpmatch (char const *response);

/* Returns true if and only if the user typed an affirmative response.  */
int
confirm (const char *message_action, const char *message_name)
{
  static FILE *confirm_file;
  static int confirm_file_EOF;
  bool status = false;

  if (!confirm_file)
    {
      if (archive == 0 || stdin_used_by)
	{
	  confirm_file = fopen (TTY_NAME, "r");
	  if (! confirm_file)
	    open_fatal (TTY_NAME);
	}
      else
	{
	  request_stdin ("-w");
	  confirm_file = stdin;
	}
    }

  fprintf (stdlis, "%s %s?", message_action, quote (message_name));
  fflush (stdlis);

  if (!confirm_file_EOF)
    {
      char *response = NULL;
      size_t response_size = 0;
      if (getline (&response, &response_size, confirm_file) < 0)
	confirm_file_EOF = 1;
      else
	status = rpmatch (response) > 0;
      free (response);
    }

  if (confirm_file_EOF)
    {
      fputc ('\n', stdlis);
      fflush (stdlis);
    }

  return status;
}

static struct fmttab {
  char const *name;
  enum archive_format fmt;
} const fmttab[] = {
  { "v7",      V7_FORMAT },
  { "oldgnu",  OLDGNU_FORMAT },
  { "ustar",   USTAR_FORMAT },
  { "posix",   POSIX_FORMAT },
#if 0 /* not fully supported yet */
  { "star",    STAR_FORMAT },
#endif
  { "gnu",     GNU_FORMAT },
  { "pax",     POSIX_FORMAT }, /* An alias for posix */
  { NULL,      0 }
};

static void
set_archive_format (char const *name)
{
  struct fmttab const *p;

  for (p = fmttab; strcmp (p->name, name) != 0; )
    if (! (++p)->name)
      USAGE_ERROR ((0, 0, _("%s: Invalid archive format"),
		    quotearg_colon (name)));

  archive_format = p->fmt;
}

static void
set_xattr_option (int value)
{
  if (value == 1)
    set_archive_format ("posix");
  xattrs_option = value;
}

const char *
archive_format_string (enum archive_format fmt)
{
  struct fmttab const *p;

  for (p = fmttab; p->name; p++)
    if (p->fmt == fmt)
      return p->name;
  return "unknown?";
}

#define FORMAT_MASK(n) (1<<(n))

static void
assert_format(unsigned fmt_mask)
{
  if ((FORMAT_MASK (archive_format) & fmt_mask) == 0)
    USAGE_ERROR ((0, 0,
		  _("GNU features wanted on incompatible archive format")));
}

const char *
subcommand_string (enum subcommand c)
{
  switch (c)
    {
    case UNKNOWN_SUBCOMMAND:
      return "unknown?";

    case APPEND_SUBCOMMAND:
      return "-r";

    case CAT_SUBCOMMAND:
      return "-A";

    case CREATE_SUBCOMMAND:
      return "-c";

    case DELETE_SUBCOMMAND:
      return "-D";

    case DIFF_SUBCOMMAND:
      return "-d";

    case EXTRACT_SUBCOMMAND:
      return "-x";

    case LIST_SUBCOMMAND:
      return "-t";

    case UPDATE_SUBCOMMAND:
      return "-u";

    case TEST_LABEL_SUBCOMMAND:
      return "--test-label";
    }
  abort ();
}

static void
tar_list_quoting_styles (struct obstack *stk, char const *prefix)
{
  int i;
  size_t prefixlen = strlen (prefix);

  for (i = 0; quoting_style_args[i]; i++)
    {
      obstack_grow (stk, prefix, prefixlen);
      obstack_grow (stk, quoting_style_args[i],
		    strlen (quoting_style_args[i]));
      obstack_1grow (stk, '\n');
    }
}

static void
tar_set_quoting_style (char *arg)
{
  int i;

  for (i = 0; quoting_style_args[i]; i++)
    if (strcmp (arg, quoting_style_args[i]) == 0)
      {
	set_quoting_style (NULL, i);
	return;
      }
  FATAL_ERROR ((0, 0,
		_("Unknown quoting style '%s'. Try '%s --quoting-style=help' to get a list."), arg, program_invocation_short_name));
}


/* Options.  */

enum
{
  ACLS_OPTION = CHAR_MAX + 1,
  ANCHORED_OPTION,
  ATIME_PRESERVE_OPTION,
  BACKUP_OPTION,
  CHECK_DEVICE_OPTION,
  CHECKPOINT_OPTION,
  CHECKPOINT_ACTION_OPTION,
  DELAY_DIRECTORY_RESTORE_OPTION,
  HARD_DEREFERENCE_OPTION,
  DELETE_OPTION,
  EXCLUDE_BACKUPS_OPTION,
  EXCLUDE_CACHES_OPTION,
  EXCLUDE_CACHES_UNDER_OPTION,
  EXCLUDE_CACHES_ALL_OPTION,
  EXCLUDE_OPTION,
  EXCLUDE_IGNORE_OPTION,
  EXCLUDE_IGNORE_RECURSIVE_OPTION,
  EXCLUDE_TAG_OPTION,
  EXCLUDE_TAG_UNDER_OPTION,
  EXCLUDE_TAG_ALL_OPTION,
  EXCLUDE_VCS_OPTION,
  EXCLUDE_VCS_IGNORES_OPTION,
  FORCE_LOCAL_OPTION,
  FULL_TIME_OPTION,
  GROUP_OPTION,
  IGNORE_CASE_OPTION,
  IGNORE_COMMAND_ERROR_OPTION,
  IGNORE_FAILED_READ_OPTION,
  INDEX_FILE_OPTION,
  KEEP_DIRECTORY_SYMLINK_OPTION,
  KEEP_NEWER_FILES_OPTION,
  LEVEL_OPTION,
  LZIP_OPTION,
  LZMA_OPTION,
  LZOP_OPTION,
  MODE_OPTION,
  MTIME_OPTION,
  NEWER_MTIME_OPTION,
  NO_ACLS_OPTION,
  NO_ANCHORED_OPTION,
  NO_AUTO_COMPRESS_OPTION,
  NO_CHECK_DEVICE_OPTION,
  NO_DELAY_DIRECTORY_RESTORE_OPTION,
  NO_IGNORE_CASE_OPTION,
  NO_IGNORE_COMMAND_ERROR_OPTION,
  NO_NULL_OPTION,
  NO_OVERWRITE_DIR_OPTION,
  NO_QUOTE_CHARS_OPTION,
  NO_RECURSION_OPTION,
  NO_SAME_OWNER_OPTION,
  NO_SAME_PERMISSIONS_OPTION,
  NO_SEEK_OPTION,
  NO_SELINUX_CONTEXT_OPTION,
  NO_UNQUOTE_OPTION,
  NO_WILDCARDS_MATCH_SLASH_OPTION,
  NO_WILDCARDS_OPTION,
  NO_XATTR_OPTION,
  NULL_OPTION,
  NUMERIC_OWNER_OPTION,
  OCCURRENCE_OPTION,
  OLD_ARCHIVE_OPTION,
  ONE_FILE_SYSTEM_OPTION,
  ONE_TOP_LEVEL_OPTION,
  OVERWRITE_DIR_OPTION,
  OVERWRITE_OPTION,
  OWNER_OPTION,
  PAX_OPTION,
  POSIX_OPTION,
  PRESERVE_OPTION,
  QUOTE_CHARS_OPTION,
  QUOTING_STYLE_OPTION,
  RECORD_SIZE_OPTION,
  RECURSION_OPTION,
  RECURSIVE_UNLINK_OPTION,
  REMOVE_FILES_OPTION,
  RESTRICT_OPTION,
  RMT_COMMAND_OPTION,
  RSH_COMMAND_OPTION,
  SAME_OWNER_OPTION,
  SELINUX_CONTEXT_OPTION,
  SHOW_DEFAULTS_OPTION,
  SHOW_OMITTED_DIRS_OPTION,
  SHOW_SNAPSHOT_FIELD_RANGES_OPTION,
  SHOW_TRANSFORMED_NAMES_OPTION,
  SKIP_OLD_FILES_OPTION,
  SORT_OPTION,
  SPARSE_VERSION_OPTION,
  STRIP_COMPONENTS_OPTION,
  SUFFIX_OPTION,
  TEST_LABEL_OPTION,
  TOTALS_OPTION,
  TO_COMMAND_OPTION,
  TRANSFORM_OPTION,
  UNQUOTE_OPTION,
  UTC_OPTION,
  VOLNO_FILE_OPTION,
  WARNING_OPTION,
  WILDCARDS_MATCH_SLASH_OPTION,
  WILDCARDS_OPTION,
  XATTR_OPTION,
  XATTR_EXCLUDE,
  XATTR_INCLUDE
};

const char *argp_program_version = "tar (" PACKAGE_NAME ") " VERSION;
const char *argp_program_bug_address = "<" PACKAGE_BUGREPORT ">";
static char const doc[] = N_("\
GNU 'tar' saves many files together into a single tape or disk archive, \
and can restore individual files from the archive.\n\
\n\
Examples:\n\
  tar -cf archive.tar foo bar  # Create archive.tar from files foo and bar.\n\
  tar -tvf archive.tar         # List all files in archive.tar verbosely.\n\
  tar -xf archive.tar          # Extract all files from archive.tar.\n")
"\v"
N_("The backup suffix is '~', unless set with --suffix or SIMPLE_BACKUP_SUFFIX.\n\
The version control may be set with --backup or VERSION_CONTROL, values are:\n\n\
  none, off       never make backups\n\
  t, numbered     make numbered backups\n\
  nil, existing   numbered if numbered backups exist, simple otherwise\n\
  never, simple   always make simple backups\n");


/* NOTE:

   Available option letters are DEQY and eqy. Consider the following
   assignments:

   [For Solaris tar compatibility =/= Is it important at all?]
   e  exit immediately with a nonzero exit status if unexpected errors occur
   E  use extended headers (--format=posix)

   [q  alias for --occurrence=1 =/= this would better be used for quiet?]

   y  per-file gzip compression
   Y  per-block gzip compression.

   Additionally, the 'n' letter is assigned for option --seek, which
   is probably not needed and should be marked as deprecated, so that
   -n may become available in the future.
*/

static struct argp_option options[] = {
#define GRID 10
  {NULL, 0, NULL, 0,
   N_("Main operation mode:"), GRID },

  {"list", 't', 0, 0,
   N_("list the contents of an archive"), GRID+1 },
  {"extract", 'x', 0, 0,
   N_("extract files from an archive"), GRID+1 },
  {"get", 0, 0, OPTION_ALIAS, NULL, GRID+1 },
  {"create", 'c', 0, 0,
   N_("create a new archive"), GRID+1 },
  {"diff", 'd', 0, 0,
   N_("find differences between archive and file system"), GRID+1 },
  {"compare", 0, 0, OPTION_ALIAS, NULL, GRID+1 },
  {"append", 'r', 0, 0,
   N_("append files to the end of an archive"), GRID+1 },
  {"update", 'u', 0, 0,
   N_("only append files newer than copy in archive"), GRID+1 },
  {"catenate", 'A', 0, 0,
   N_("append tar files to an archive"), GRID+1 },
  {"concatenate", 0, 0, OPTION_ALIAS, NULL, GRID+1 },
  {"delete", DELETE_OPTION, 0, 0,
   N_("delete from the archive (not on mag tapes!)"), GRID+1 },
  {"test-label", TEST_LABEL_OPTION, NULL, 0,
   N_("test the archive volume label and exit"), GRID+1 },
#undef GRID

#define GRID 20
  {NULL, 0, NULL, 0,
   N_("Operation modifiers:"), GRID },

  {"sparse", 'S', 0, 0,
   N_("handle sparse files efficiently"), GRID+1 },
  {"sparse-version", SPARSE_VERSION_OPTION, N_("MAJOR[.MINOR]"), 0,
   N_("set version of the sparse format to use (implies --sparse)"), GRID+1},
  {"incremental", 'G', 0, 0,
   N_("handle old GNU-format incremental backup"), GRID+1 },
  {"listed-incremental", 'g', N_("FILE"), 0,
   N_("handle new GNU-format incremental backup"), GRID+1 },
  {"level", LEVEL_OPTION, N_("NUMBER"), 0,
   N_("dump level for created listed-incremental archive"), GRID+1 },
  {"ignore-failed-read", IGNORE_FAILED_READ_OPTION, 0, 0,
   N_("do not exit with nonzero on unreadable files"), GRID+1 },
  {"occurrence", OCCURRENCE_OPTION, N_("NUMBER"), OPTION_ARG_OPTIONAL,
   N_("process only the NUMBERth occurrence of each file in the archive;"
      " this option is valid only in conjunction with one of the subcommands"
      " --delete, --diff, --extract or --list and when a list of files"
      " is given either on the command line or via the -T option;"
      " NUMBER defaults to 1"), GRID+1 },
  {"seek", 'n', NULL, 0,
   N_("archive is seekable"), GRID+1 },
  {"no-seek", NO_SEEK_OPTION, NULL, 0,
   N_("archive is not seekable"), GRID+1 },
  {"no-check-device", NO_CHECK_DEVICE_OPTION, NULL, 0,
   N_("do not check device numbers when creating incremental archives"),
   GRID+1 },
  {"check-device", CHECK_DEVICE_OPTION, NULL, 0,
   N_("check device numbers when creating incremental archives (default)"),
   GRID+1 },
#undef GRID

#define GRID 30
  {NULL, 0, NULL, 0,
   N_("Overwrite control:"), GRID },

  {"verify", 'W', 0, 0,
   N_("attempt to verify the archive after writing it"), GRID+1 },
  {"remove-files", REMOVE_FILES_OPTION, 0, 0,
   N_("remove files after adding them to the archive"), GRID+1 },
  {"keep-old-files", 'k', 0, 0,
   N_("don't replace existing files when extracting, "
      "treat them as errors"), GRID+1 },
  {"skip-old-files", SKIP_OLD_FILES_OPTION, 0, 0,
   N_("don't replace existing files when extracting, silently skip over them"),
   GRID+1 },
  {"keep-newer-files", KEEP_NEWER_FILES_OPTION, 0, 0,
   N_("don't replace existing files that are newer than their archive copies"), GRID+1 },
  {"overwrite", OVERWRITE_OPTION, 0, 0,
   N_("overwrite existing files when extracting"), GRID+1 },
  {"unlink-first", 'U', 0, 0,
   N_("remove each file prior to extracting over it"), GRID+1 },
  {"recursive-unlink", RECURSIVE_UNLINK_OPTION, 0, 0,
   N_("empty hierarchies prior to extracting directory"), GRID+1 },
  {"no-overwrite-dir", NO_OVERWRITE_DIR_OPTION, 0, 0,
   N_("preserve metadata of existing directories"), GRID+1 },
  {"overwrite-dir", OVERWRITE_DIR_OPTION, 0, 0,
   N_("overwrite metadata of existing directories when extracting (default)"),
   GRID+1 },
  {"keep-directory-symlink", KEEP_DIRECTORY_SYMLINK_OPTION, 0, 0,
   N_("preserve existing symlinks to directories when extracting"),
   GRID+1 },
  {"one-top-level", ONE_TOP_LEVEL_OPTION, N_("DIR"), OPTION_ARG_OPTIONAL,
   N_("create a subdirectory to avoid having loose files extracted"),
   GRID+1 },
#undef GRID

#define GRID 40
  {NULL, 0, NULL, 0,
   N_("Select output stream:"), GRID },

  {"to-stdout", 'O', 0, 0,
   N_("extract files to standard output"), GRID+1 },
  {"to-command", TO_COMMAND_OPTION, N_("COMMAND"), 0,
   N_("pipe extracted files to another program"), GRID+1 },
  {"ignore-command-error", IGNORE_COMMAND_ERROR_OPTION, 0, 0,
   N_("ignore exit codes of children"), GRID+1 },
  {"no-ignore-command-error", NO_IGNORE_COMMAND_ERROR_OPTION, 0, 0,
   N_("treat non-zero exit codes of children as error"), GRID+1 },
#undef GRID

#define GRID 50
  {NULL, 0, NULL, 0,
   N_("Handling of file attributes:"), GRID },

  {"owner", OWNER_OPTION, N_("NAME"), 0,
   N_("force NAME as owner for added files"), GRID+1 },
  {"group", GROUP_OPTION, N_("NAME"), 0,
   N_("force NAME as group for added files"), GRID+1 },
  {"mtime", MTIME_OPTION, N_("DATE-OR-FILE"), 0,
   N_("set mtime for added files from DATE-OR-FILE"), GRID+1 },
  {"mode", MODE_OPTION, N_("CHANGES"), 0,
   N_("force (symbolic) mode CHANGES for added files"), GRID+1 },
  {"atime-preserve", ATIME_PRESERVE_OPTION,
   N_("METHOD"), OPTION_ARG_OPTIONAL,
   N_("preserve access times on dumped files, either by restoring the times"
      " after reading (METHOD='replace'; default) or by not setting the times"
      " in the first place (METHOD='system')"), GRID+1 },
  {"touch", 'm', 0, 0,
   N_("don't extract file modified time"), GRID+1 },
  {"same-owner", SAME_OWNER_OPTION, 0, 0,
   N_("try extracting files with the same ownership as exists in the archive (default for superuser)"), GRID+1 },
  {"no-same-owner", NO_SAME_OWNER_OPTION, 0, 0,
   N_("extract files as yourself (default for ordinary users)"), GRID+1 },
  {"numeric-owner", NUMERIC_OWNER_OPTION, 0, 0,
   N_("always use numbers for user/group names"), GRID+1 },
  {"preserve-permissions", 'p', 0, 0,
   N_("extract information about file permissions (default for superuser)"),
   GRID+1 },
  {"same-permissions", 0, 0, OPTION_ALIAS, NULL, GRID+1 },
  {"no-same-permissions", NO_SAME_PERMISSIONS_OPTION, 0, 0,
   N_("apply the user's umask when extracting permissions from the archive (default for ordinary users)"), GRID+1 },
  {"preserve-order", 's', 0, 0,
   N_("member arguments are listed in the same order as the "
      "files in the archive"), GRID+1 },
  {"same-order", 0, 0, OPTION_ALIAS, NULL, GRID+1 },
  {"preserve", PRESERVE_OPTION, 0, 0,
   N_("same as both -p and -s"), GRID+1 },
  {"delay-directory-restore", DELAY_DIRECTORY_RESTORE_OPTION, 0, 0,
   N_("delay setting modification times and permissions of extracted"
      " directories until the end of extraction"), GRID+1 },
  {"no-delay-directory-restore", NO_DELAY_DIRECTORY_RESTORE_OPTION, 0, 0,
   N_("cancel the effect of --delay-directory-restore option"), GRID+1 },
  {"sort", SORT_OPTION, N_("ORDER"), 0,
#if D_INO_IN_DIRENT
   N_("directory sorting order: none (default), name or inode"
#else
   N_("directory sorting order: none (default) or name"
#endif
     ), GRID+1 },
#undef GRID

#define GRID 55
  {NULL, 0, NULL, 0,
   N_("Handling of extended file attributes:"), GRID },

  {"xattrs", XATTR_OPTION, 0, 0,
   N_("Enable extended attributes support"), GRID+1 },
  {"no-xattrs", NO_XATTR_OPTION, 0, 0,
   N_("Disable extended attributes support"), GRID+1 },
  {"xattrs-include", XATTR_INCLUDE, N_("MASK"), 0,
   N_("specify the include pattern for xattr keys"), GRID+1 },
  {"xattrs-exclude", XATTR_EXCLUDE, N_("MASK"), 0,
   N_("specify the exclude pattern for xattr keys"), GRID+1 },
  {"selinux", SELINUX_CONTEXT_OPTION, 0, 0,
   N_("Enable the SELinux context support"), GRID+1 },
  {"no-selinux", NO_SELINUX_CONTEXT_OPTION, 0, 0,
   N_("Disable the SELinux context support"), GRID+1 },
  {"acls", ACLS_OPTION, 0, 0,
   N_("Enable the POSIX ACLs support"), GRID+1 },
  {"no-acls", NO_ACLS_OPTION, 0, 0,
   N_("Disable the POSIX ACLs support"), GRID+1 },
#undef GRID

#define GRID 60
  {NULL, 0, NULL, 0,
   N_("Device selection and switching:"), GRID },

  {"file", 'f', N_("ARCHIVE"), 0,
   N_("use archive file or device ARCHIVE"), GRID+1 },
  {"force-local", FORCE_LOCAL_OPTION, 0, 0,
   N_("archive file is local even if it has a colon"), GRID+1 },
  {"rmt-command", RMT_COMMAND_OPTION, N_("COMMAND"), 0,
   N_("use given rmt COMMAND instead of rmt"), GRID+1 },
  {"rsh-command", RSH_COMMAND_OPTION, N_("COMMAND"), 0,
   N_("use remote COMMAND instead of rsh"), GRID+1 },
#ifdef DEVICE_PREFIX
  {"-[0-7][lmh]", 0, NULL, OPTION_DOC, /* It is OK, since 'name' will never be
					  translated */
   N_("specify drive and density"), GRID+1 },
#endif
  {NULL, '0', NULL, OPTION_HIDDEN, NULL, GRID+1 },
  {NULL, '1', NULL, OPTION_HIDDEN, NULL, GRID+1 },
  {NULL, '2', NULL, OPTION_HIDDEN, NULL, GRID+1 },
  {NULL, '3', NULL, OPTION_HIDDEN, NULL, GRID+1 },
  {NULL, '4', NULL, OPTION_HIDDEN, NULL, GRID+1 },
  {NULL, '5', NULL, OPTION_HIDDEN, NULL, GRID+1 },
  {NULL, '6', NULL, OPTION_HIDDEN, NULL, GRID+1 },
  {NULL, '7', NULL, OPTION_HIDDEN, NULL, GRID+1 },
  {NULL, '8', NULL, OPTION_HIDDEN, NULL, GRID+1 },
  {NULL, '9', NULL, OPTION_HIDDEN, NULL, GRID+1 },

  {"multi-volume", 'M', 0, 0,
   N_("create/list/extract multi-volume archive"), GRID+1 },
  {"tape-length", 'L', N_("NUMBER"), 0,
   N_("change tape after writing NUMBER x 1024 bytes"), GRID+1 },
  {"info-script", 'F', N_("NAME"), 0,
   N_("run script at end of each tape (implies -M)"), GRID+1 },
  {"new-volume-script", 0, 0, OPTION_ALIAS, NULL, GRID+1 },
  {"volno-file", VOLNO_FILE_OPTION, N_("FILE"), 0,
   N_("use/update the volume number in FILE"), GRID+1 },
#undef GRID

#define GRID 70
  {NULL, 0, NULL, 0,
   N_("Device blocking:"), GRID },

  {"blocking-factor", 'b', N_("BLOCKS"), 0,
   N_("BLOCKS x 512 bytes per record"), GRID+1 },
  {"record-size", RECORD_SIZE_OPTION, N_("NUMBER"), 0,
   N_("NUMBER of bytes per record, multiple of 512"), GRID+1 },
  {"ignore-zeros", 'i', 0, 0,
   N_("ignore zeroed blocks in archive (means EOF)"), GRID+1 },
  {"read-full-records", 'B', 0, 0,
   N_("reblock as we read (for 4.2BSD pipes)"), GRID+1 },
#undef GRID

#define GRID 80
  {NULL, 0, NULL, 0,
   N_("Archive format selection:"), GRID },

  {"format", 'H', N_("FORMAT"), 0,
   N_("create archive of the given format"), GRID+1 },

  {NULL, 0, NULL, 0, N_("FORMAT is one of the following:"), GRID+2 },
  {"  v7", 0, NULL, OPTION_DOC|OPTION_NO_TRANS, N_("old V7 tar format"),
   GRID+3 },
  {"  oldgnu", 0, NULL, OPTION_DOC|OPTION_NO_TRANS,
   N_("GNU format as per tar <= 1.12"), GRID+3 },
  {"  gnu", 0, NULL, OPTION_DOC|OPTION_NO_TRANS,
   N_("GNU tar 1.13.x format"), GRID+3 },
  {"  ustar", 0, NULL, OPTION_DOC|OPTION_NO_TRANS,
   N_("POSIX 1003.1-1988 (ustar) format"), GRID+3 },
  {"  pax", 0, NULL, OPTION_DOC|OPTION_NO_TRANS,
   N_("POSIX 1003.1-2001 (pax) format"), GRID+3 },
  {"  posix", 0, NULL, OPTION_DOC|OPTION_NO_TRANS, N_("same as pax"), GRID+3 },

  {"old-archive", OLD_ARCHIVE_OPTION, 0, 0, /* FIXME */
   N_("same as --format=v7"), GRID+8 },
  {"portability", 0, 0, OPTION_ALIAS, NULL, GRID+8 },
  {"posix", POSIX_OPTION, 0, 0,
   N_("same as --format=posix"), GRID+8 },
  {"pax-option", PAX_OPTION, N_("keyword[[:]=value][,keyword[[:]=value]]..."), 0,
   N_("control pax keywords"), GRID+8 },
  {"label", 'V', N_("TEXT"), 0,
   N_("create archive with volume name TEXT; at list/extract time, use TEXT as a globbing pattern for volume name"), GRID+8 },
#undef GRID

#define GRID 90
  {NULL, 0, NULL, 0,
   N_("Compression options:"), GRID },
  {"auto-compress", 'a', 0, 0,
   N_("use archive suffix to determine the compression program"), GRID+1 },
  {"no-auto-compress", NO_AUTO_COMPRESS_OPTION, 0, 0,
   N_("do not use archive suffix to determine the compression program"),
   GRID+1 },
  {"use-compress-program", 'I', N_("PROG"), 0,
   N_("filter through PROG (must accept -d)"), GRID+1 },
  /* Note: docstrings for the options below are generated by tar_help_filter */
  {"bzip2", 'j', 0, 0, NULL, GRID+1 },
  {"gzip", 'z', 0, 0, NULL, GRID+1 },
  {"gunzip", 0, 0, OPTION_ALIAS, NULL, GRID+1 },
  {"ungzip", 0, 0, OPTION_ALIAS, NULL, GRID+1 },
  {"compress", 'Z', 0, 0, NULL, GRID+1 },
  {"uncompress", 0, 0, OPTION_ALIAS, NULL, GRID+1 },
  {"lzip", LZIP_OPTION, 0, 0, NULL, GRID+1 },
  {"lzma", LZMA_OPTION, 0, 0, NULL, GRID+1 },
  {"lzop", LZOP_OPTION, 0, 0, NULL, GRID+1 },
  {"xz", 'J', 0, 0, NULL, GRID+1 },
#undef GRID

#define GRID 100
  {NULL, 0, NULL, 0,
   N_("Local file selection:"), GRID },

  {"add-file", ARGP_KEY_ARG, N_("FILE"), 0,
   N_("add given FILE to the archive (useful if its name starts with a dash)"), GRID+1 },
  {"directory", 'C', N_("DIR"), 0,
   N_("change to directory DIR"), GRID+1 },
  {"files-from", 'T', N_("FILE"), 0,
   N_("get names to extract or create from FILE"), GRID+1 },
  {"null", NULL_OPTION, 0, 0,
   N_("-T reads null-terminated names, disable -C"), GRID+1 },
  {"no-null", NO_NULL_OPTION, 0, 0,
   N_("disable the effect of the previous --null option"), GRID+1 },
  {"unquote", UNQUOTE_OPTION, 0, 0,
   N_("unquote input file or member names (default)"), GRID+1 },
  {"no-unquote", NO_UNQUOTE_OPTION, 0, 0,
   N_("do not unquote input file or member names"), GRID+1 },
  {"exclude", EXCLUDE_OPTION, N_("PATTERN"), 0,
   N_("exclude files, given as a PATTERN"), GRID+1 },
  {"exclude-from", 'X', N_("FILE"), 0,
   N_("exclude patterns listed in FILE"), GRID+1 },
  {"exclude-caches", EXCLUDE_CACHES_OPTION, 0, 0,
   N_("exclude contents of directories containing CACHEDIR.TAG, "
      "except for the tag file itself"), GRID+1 },
  {"exclude-caches-under", EXCLUDE_CACHES_UNDER_OPTION, 0, 0,
   N_("exclude everything under directories containing CACHEDIR.TAG"),
   GRID+1 },
  {"exclude-caches-all", EXCLUDE_CACHES_ALL_OPTION, 0, 0,
   N_("exclude directories containing CACHEDIR.TAG"), GRID+1 },
  {"exclude-tag", EXCLUDE_TAG_OPTION, N_("FILE"), 0,
   N_("exclude contents of directories containing FILE, except"
      " for FILE itself"), GRID+1 },
  {"exclude-ignore", EXCLUDE_IGNORE_OPTION, N_("FILE"), 0,
    N_("read exclude patterns for each directory from FILE, if it exists"),
   GRID+1 },
  {"exclude-ignore-recursive", EXCLUDE_IGNORE_RECURSIVE_OPTION, N_("FILE"), 0,
    N_("read exclude patterns for each directory and its subdirectories "
       "from FILE, if it exists"), GRID+1 },
  {"exclude-tag-under", EXCLUDE_TAG_UNDER_OPTION, N_("FILE"), 0,
   N_("exclude everything under directories containing FILE"), GRID+1 },
  {"exclude-tag-all", EXCLUDE_TAG_ALL_OPTION, N_("FILE"), 0,
   N_("exclude directories containing FILE"), GRID+1 },
  {"exclude-vcs", EXCLUDE_VCS_OPTION, NULL, 0,
   N_("exclude version control system directories"), GRID+1 },
  {"exclude-vcs-ignores", EXCLUDE_VCS_IGNORES_OPTION, NULL, 0,
   N_("read exclude patterns from the VCS ignore files"), GRID+1 },
  {"exclude-backups", EXCLUDE_BACKUPS_OPTION, NULL, 0,
   N_("exclude backup and lock files"), GRID+1 },
  {"no-recursion", NO_RECURSION_OPTION, 0, 0,
   N_("avoid descending automatically in directories"), GRID+1 },
  {"one-file-system", ONE_FILE_SYSTEM_OPTION, 0, 0,
   N_("stay in local file system when creating archive"), GRID+1 },
  {"recursion", RECURSION_OPTION, 0, 0,
   N_("recurse into directories (default)"), GRID+1 },
  {"absolute-names", 'P', 0, 0,
   N_("don't strip leading '/'s from file names"), GRID+1 },
  {"dereference", 'h', 0, 0,
   N_("follow symlinks; archive and dump the files they point to"), GRID+1 },
  {"hard-dereference", HARD_DEREFERENCE_OPTION, 0, 0,
   N_("follow hard links; archive and dump the files they refer to"), GRID+1 },
  {"starting-file", 'K', N_("MEMBER-NAME"), 0,
   N_("begin at member MEMBER-NAME when reading the archive"), GRID+1 },
  {"newer", 'N', N_("DATE-OR-FILE"), 0,
   N_("only store files newer than DATE-OR-FILE"), GRID+1 },
  {"after-date", 0, 0, OPTION_ALIAS, NULL, GRID+1 },
  {"newer-mtime", NEWER_MTIME_OPTION, N_("DATE"), 0,
   N_("compare date and time when data changed only"), GRID+1 },
  {"backup", BACKUP_OPTION, N_("CONTROL"), OPTION_ARG_OPTIONAL,
   N_("backup before removal, choose version CONTROL"), GRID+1 },
  {"suffix", SUFFIX_OPTION, N_("STRING"), 0,
   N_("backup before removal, override usual suffix ('~' unless overridden by environment variable SIMPLE_BACKUP_SUFFIX)"), GRID+1 },
#undef GRID

#define GRID 110
  {NULL, 0, NULL, 0,
   N_("File name transformations:"), GRID },
  {"strip-components", STRIP_COMPONENTS_OPTION, N_("NUMBER"), 0,
   N_("strip NUMBER leading components from file names on extraction"),
   GRID+1 },
  {"transform", TRANSFORM_OPTION, N_("EXPRESSION"), 0,
   N_("use sed replace EXPRESSION to transform file names"), GRID+1 },
  {"xform", 0, 0, OPTION_ALIAS, NULL, GRID+1 },
#undef GRID

#define GRID 120
  {NULL, 0, NULL, 0,
   N_("File name matching options (affect both exclude and include patterns):"),
   GRID },
  {"ignore-case", IGNORE_CASE_OPTION, 0, 0,
   N_("ignore case"), GRID+1 },
  {"anchored", ANCHORED_OPTION, 0, 0,
   N_("patterns match file name start"), GRID+1 },
  {"no-anchored", NO_ANCHORED_OPTION, 0, 0,
   N_("patterns match after any '/' (default for exclusion)"), GRID+1 },
  {"no-ignore-case", NO_IGNORE_CASE_OPTION, 0, 0,
   N_("case sensitive matching (default)"), GRID+1 },
  {"wildcards", WILDCARDS_OPTION, 0, 0,
   N_("use wildcards (default for exclusion)"), GRID+1 },
  {"no-wildcards", NO_WILDCARDS_OPTION, 0, 0,
   N_("verbatim string matching"), GRID+1 },
  {"no-wildcards-match-slash", NO_WILDCARDS_MATCH_SLASH_OPTION, 0, 0,
   N_("wildcards do not match '/'"), GRID+1 },
  {"wildcards-match-slash", WILDCARDS_MATCH_SLASH_OPTION, 0, 0,
   N_("wildcards match '/' (default for exclusion)"), GRID+1 },
#undef GRID

#define GRID 130
  {NULL, 0, NULL, 0,
   N_("Informative output:"), GRID },

  {"verbose", 'v', 0, 0,
   N_("verbosely list files processed"), GRID+1 },
  {"warning", WARNING_OPTION, N_("KEYWORD"), 0,
   N_("warning control"), GRID+1 },
  {"checkpoint", CHECKPOINT_OPTION, N_("NUMBER"), OPTION_ARG_OPTIONAL,
   N_("display progress messages every NUMBERth record (default 10)"),
   GRID+1 },
  {"checkpoint-action", CHECKPOINT_ACTION_OPTION, N_("ACTION"), 0,
   N_("execute ACTION on each checkpoint"),
   GRID+1 },
  {"check-links", 'l', 0, 0,
   N_("print a message if not all links are dumped"), GRID+1 },
  {"totals", TOTALS_OPTION, N_("SIGNAL"), OPTION_ARG_OPTIONAL,
   N_("print total bytes after processing the archive; "
      "with an argument - print total bytes when this SIGNAL is delivered; "
      "Allowed signals are: SIGHUP, SIGQUIT, SIGINT, SIGUSR1 and SIGUSR2; "
      "the names without SIG prefix are also accepted"), GRID+1 },
  {"utc", UTC_OPTION, 0, 0,
   N_("print file modification times in UTC"), GRID+1 },
  {"full-time", FULL_TIME_OPTION, 0, 0,
   N_("print file time to its full resolution"), GRID+1 },
  {"index-file", INDEX_FILE_OPTION, N_("FILE"), 0,
   N_("send verbose output to FILE"), GRID+1 },
  {"block-number", 'R', 0, 0,
   N_("show block number within archive with each message"), GRID+1 },
  {"interactive", 'w', 0, 0,
   N_("ask for confirmation for every action"), GRID+1 },
  {"confirmation", 0, 0, OPTION_ALIAS, NULL, GRID+1 },
  {"show-defaults", SHOW_DEFAULTS_OPTION, 0, 0,
   N_("show tar defaults"), GRID+1 },
  {"show-snapshot-field-ranges", SHOW_SNAPSHOT_FIELD_RANGES_OPTION, 0, 0,
   N_("show valid ranges for snapshot-file fields"), GRID+1 },
  {"show-omitted-dirs", SHOW_OMITTED_DIRS_OPTION, 0, 0,
   N_("when listing or extracting, list each directory that does not match search criteria"), GRID+1 },
  {"show-transformed-names", SHOW_TRANSFORMED_NAMES_OPTION, 0, 0,
   N_("show file or archive names after transformation"),
   GRID+1 },
  {"show-stored-names", 0, 0, OPTION_ALIAS, NULL, GRID+1 },
  {"quoting-style", QUOTING_STYLE_OPTION, N_("STYLE"), 0,
   N_("set name quoting style; see below for valid STYLE values"), GRID+1 },
  {"quote-chars", QUOTE_CHARS_OPTION, N_("STRING"), 0,
   N_("additionally quote characters from STRING"), GRID+1 },
  {"no-quote-chars", NO_QUOTE_CHARS_OPTION, N_("STRING"), 0,
   N_("disable quoting for characters from STRING"), GRID+1 },
#undef GRID

#define GRID 140
  {NULL, 0, NULL, 0,
   N_("Compatibility options:"), GRID },

  {NULL, 'o', 0, 0,
   N_("when creating, same as --old-archive; when extracting, same as --no-same-owner"), GRID+1 },
#undef GRID

#define GRID 150
  {NULL, 0, NULL, 0,
   N_("Other options:"), GRID },

  {"restrict", RESTRICT_OPTION, 0, 0,
   N_("disable use of some potentially harmful options"), -1 },
#undef GRID

  {0, 0, 0, 0, 0, 0}
};

static char const *const atime_preserve_args[] =
{
  "replace", "system", NULL
};

static enum atime_preserve const atime_preserve_types[] =
{
  replace_atime_preserve, system_atime_preserve
};

/* Make sure atime_preserve_types has as much entries as atime_preserve_args
   (minus 1 for NULL guard) */
ARGMATCH_VERIFY (atime_preserve_args, atime_preserve_types);

/* Wildcard matching settings */
enum wildcards
  {
    default_wildcards, /* For exclusion == enable_wildcards,
			  for inclusion == disable_wildcards */
    disable_wildcards,
    enable_wildcards
  };

struct tar_args        /* Variables used during option parsing */
{
  struct textual_date *textual_date; /* Keeps the arguments to --newer-mtime
					and/or --date option if they are
					textual dates */
  enum wildcards wildcards;        /* Wildcard settings (--wildcards/
				      --no-wildcards) */
  int matching_flags;              /* exclude_fnmatch options */
  int include_anchored;            /* Pattern anchoring options used for
				      file inclusion */
  bool o_option;                   /* True if -o option was given */
  bool pax_option;                 /* True if --pax-option was given */
  char const *backup_suffix_string;   /* --suffix option argument */
  char const *version_control_string; /* --backup option argument */
  bool input_files;                /* True if some input files where given */
  int compress_autodetect;         /* True if compression autodetection should
				      be attempted when creating archives */
};


#define MAKE_EXCL_OPTIONS(args) \
 ((((args)->wildcards != disable_wildcards) ? EXCLUDE_WILDCARDS : 0) \
  | (args)->matching_flags \
  | recursion_option)

#define MAKE_INCL_OPTIONS(args) \
 ((((args)->wildcards == enable_wildcards) ? EXCLUDE_WILDCARDS : 0) \
  | (args)->include_anchored \
  | (args)->matching_flags \
  | recursion_option)

static char const * const vcs_file_table[] = {
  /* CVS: */
  "CVS",
  ".cvsignore",
  /* RCS: */
  "RCS",
  /* SCCS: */
  "SCCS",
  /* SVN: */
  ".svn",
  /* git: */
  ".git",
  ".gitignore",
  /* Arch: */
  ".arch-ids",
  "{arch}",
  "=RELEASE-ID",
  "=meta-update",
  "=update",
  /* Bazaar */
  ".bzr",
  ".bzrignore",
  ".bzrtags",
  /* Mercurial */
  ".hg",
  ".hgignore",
  ".hgtags",
  /* darcs */
  "_darcs",
  NULL
};

static char const * const backup_file_table[] = {
  ".#*",
  "*~",
  "#*#",
  NULL
};

static void
add_exclude_array (char const * const * fv, int opts)
{
  int i;

  for (i = 0; fv[i]; i++)
    add_exclude (excluded, fv[i], opts);
}


static char *
format_default_settings (void)
{
  return xasprintf (
	    "--format=%s -f%s -b%d --quoting-style=%s --rmt-command=%s"
#ifdef REMOTE_SHELL
	    " --rsh-command=%s"
#endif
	    ,
	    archive_format_string (DEFAULT_ARCHIVE_FORMAT),
	    DEFAULT_ARCHIVE, DEFAULT_BLOCKING,
	    quoting_style_args[DEFAULT_QUOTING_STYLE],
	    DEFAULT_RMT_COMMAND
#ifdef REMOTE_SHELL
	    , REMOTE_SHELL
#endif
	    );
}


static void
set_subcommand_option (enum subcommand subcommand)
{
  if (subcommand_option != UNKNOWN_SUBCOMMAND
      && subcommand_option != subcommand)
    USAGE_ERROR ((0, 0,
		  _("You may not specify more than one '-Acdtrux', '--delete' or  '--test-label' option")));

  subcommand_option = subcommand;
}

static void
set_use_compress_program_option (const char *string)
{
  if (use_compress_program_option
      && strcmp (use_compress_program_option, string) != 0)
    USAGE_ERROR ((0, 0, _("Conflicting compression options")));

  use_compress_program_option = string;
}

static void
sigstat (int signo)
{
  compute_duration ();
  print_total_stats ();
#ifndef HAVE_SIGACTION
  signal (signo, sigstat);
#endif
}

static void
stat_on_signal (int signo)
{
#ifdef HAVE_SIGACTION
# ifndef SA_RESTART
#  define SA_RESTART 0
# endif
  struct sigaction act;
  act.sa_handler = sigstat;
  sigemptyset (&act.sa_mask);
  act.sa_flags = SA_RESTART;
  sigaction (signo, &act, NULL);
#else
  signal (signo, sigstat);
#endif
}

static void
set_stat_signal (const char *name)
{
  static struct sigtab
  {
    char const *name;
    int signo;
  } const sigtab[] = {
    { "SIGUSR1", SIGUSR1 },
    { "USR1", SIGUSR1 },
    { "SIGUSR2", SIGUSR2 },
    { "USR2", SIGUSR2 },
    { "SIGHUP", SIGHUP },
    { "HUP", SIGHUP },
    { "SIGINT", SIGINT },
    { "INT", SIGINT },
    { "SIGQUIT", SIGQUIT },
    { "QUIT", SIGQUIT }
  };
  struct sigtab const *p;

  for (p = sigtab; p < sigtab + sizeof (sigtab) / sizeof (sigtab[0]); p++)
    if (strcmp (p->name, name) == 0)
      {
	stat_on_signal (p->signo);
	return;
      }
  FATAL_ERROR ((0, 0, _("Unknown signal name: %s"), name));
}


struct textual_date
{
  struct textual_date *next;
  struct timespec ts;
  const char *option;
  char *date;
};

static int
get_date_or_file (struct tar_args *args, const char *option,
		  const char *str, struct timespec *ts)
{
  if (FILE_SYSTEM_PREFIX_LEN (str) != 0
      || ISSLASH (*str)
      || *str == '.')
    {
      struct stat st;
      if (stat (str, &st) != 0)
	{
	  stat_error (str);
	  USAGE_ERROR ((0, 0, _("Date sample file not found")));
	}
      *ts = get_stat_mtime (&st);
    }
  else
    {
      if (! parse_datetime (ts, str, NULL))
	{
	  WARN ((0, 0, _("Substituting %s for unknown date format %s"),
		 tartime (*ts, false), quote (str)));
	  ts->tv_nsec = 0;
	  return 1;
	}
      else
	{
	  struct textual_date *p = xmalloc (sizeof (*p));
	  p->ts = *ts;
	  p->option = option;
	  p->date = xstrdup (str);
	  p->next = args->textual_date;
	  args->textual_date = p;
	}
    }
  return 0;
}

static void
report_textual_dates (struct tar_args *args)
{
  struct textual_date *p;
  for (p = args->textual_date; p; )
    {
      struct textual_date *next = p->next;
      if (verbose_option)
	{
	  char const *treated_as = tartime (p->ts, true);
	  if (strcmp (p->date, treated_as) != 0)
	    WARN ((0, 0, _("Option %s: Treating date '%s' as %s"),
		   p->option, p->date, treated_as));
	}
      free (p->date);
      free (p);
      p = next;
    }
}


static bool files_from_option;  /* When set, tar will not refuse to create
				   empty archives */

/* Default density numbers for [0-9][lmh] device specifications */

#if defined DEVICE_PREFIX && !defined DENSITY_LETTER
# ifndef LOW_DENSITY_NUM
#  define LOW_DENSITY_NUM 0
# endif

# ifndef MID_DENSITY_NUM
#  define MID_DENSITY_NUM 8
# endif

# ifndef HIGH_DENSITY_NUM
#  define HIGH_DENSITY_NUM 16
# endif
#endif


static char *
tar_help_filter (int key, const char *text, void *input)
{
  struct obstack stk;
  char *s;

  switch (key)
    {
    default:
      s = (char*) text;
      break;

    case 'j':
      s = xasprintf (_("filter the archive through %s"), BZIP2_PROGRAM);
      break;

    case 'z':
      s = xasprintf (_("filter the archive through %s"), GZIP_PROGRAM);
      break;

    case 'Z':
      s = xasprintf (_("filter the archive through %s"), COMPRESS_PROGRAM);
      break;

    case LZIP_OPTION:
      s = xasprintf (_("filter the archive through %s"), LZIP_PROGRAM);
      break;

    case LZMA_OPTION:
      s = xasprintf (_("filter the archive through %s"), LZMA_PROGRAM);
      break;

    case LZOP_OPTION:
      s = xasprintf (_("filter the archive through %s"), LZOP_PROGRAM);

    case 'J':
      s = xasprintf (_("filter the archive through %s"), XZ_PROGRAM);
      break;

    case ARGP_KEY_HELP_EXTRA:
      {
	const char *tstr;

	obstack_init (&stk);
	tstr = _("Valid arguments for the --quoting-style option are:");
	obstack_grow (&stk, tstr, strlen (tstr));
	obstack_grow (&stk, "\n\n", 2);
	tar_list_quoting_styles (&stk, "  ");
	tstr = _("\n*This* tar defaults to:\n");
	obstack_grow (&stk, tstr, strlen (tstr));
	s = format_default_settings ();
	obstack_grow (&stk, s, strlen (s));
	obstack_1grow (&stk, '\n');
	obstack_1grow (&stk, 0);
	s = xstrdup (obstack_finish (&stk));
	obstack_free (&stk, NULL);
      }
    }
  return s;
}

static char *
expand_pax_option (struct tar_args *targs, const char *arg)
{
  struct obstack stk;
  char *res;

  obstack_init (&stk);
  while (*arg)
    {
      size_t seglen = strcspn (arg, ",");
      char *p = memchr (arg, '=', seglen);
      if (p)
	{
	  size_t len = p - arg + 1;
	  obstack_grow (&stk, arg, len);
	  len = seglen - len;
	  for (++p; *p && isspace ((unsigned char) *p); p++)
	    len--;
	  if (*p == '{' && p[len-1] == '}')
	    {
	      struct timespec ts;
	      char *tmp = xmalloc (len);
	      memcpy (tmp, p + 1, len-2);
	      tmp[len-2] = 0;
	      if (get_date_or_file (targs, "--pax-option", tmp, &ts) == 0)
		{
		  char buf[TIMESPEC_STRSIZE_BOUND];
		  char const *s = code_timespec (ts, buf);
		  obstack_grow (&stk, s, strlen (s));
		}
	      else
		obstack_grow (&stk, p, len);
	      free (tmp);
	    }
	  else
	    obstack_grow (&stk, p, len);
	}
      else
	obstack_grow (&stk, arg, seglen);

      arg += seglen;
      if (*arg)
	{
	  obstack_1grow (&stk, *arg);
	  arg++;
	}
    }
  obstack_1grow (&stk, 0);
  res = xstrdup (obstack_finish (&stk));
  obstack_free (&stk, NULL);
  return res;
}


static uintmax_t
parse_owner_group (char *arg, uintmax_t field_max, char const **name_option)
{
  uintmax_t u = UINTMAX_MAX;
  char *end;
  char const *name = 0;
  char const *invalid_num = 0;
  char *colon = strchr (arg, ':');

  if (colon)
    {
      char const *num = colon + 1;
      *colon = '\0';
      if (*arg)
	name = arg;
      if (num && (! (xstrtoumax (num, &end, 10, &u, "") == LONGINT_OK
		     && u <= field_max)))
	invalid_num = num;
    }
  else
    {
      uintmax_t u1;
      switch ('0' <= *arg && *arg <= '9'
	      ? xstrtoumax (arg, &end, 10, &u1, "")
	      : LONGINT_INVALID)
	{
	default:
	  name = arg;
	  break;

	case LONGINT_OK:
	  if (u1 <= field_max)
	    {
	      u = u1;
	      break;
	    }
	  /* Fall through.  */
	case LONGINT_OVERFLOW:
	  invalid_num = arg;
	  break;
	}
    }

  if (invalid_num)
    FATAL_ERROR ((0, 0, "%s: %s", quotearg_colon (invalid_num),
		  _("Invalid owner or group ID")));
  if (name)
    *name_option = name;
  return u;
}

#define TAR_SIZE_SUFFIXES "bBcGgkKMmPTtw"

/* Either NL or NUL, as decided by the --null option.  */
static char filename_terminator;

static char const *const sort_mode_arg[] = {
  "none",
  "name",
  //"inode",
  NULL
};

static int sort_mode_flag[] = {
    SAVEDIR_SORT_NONE,
    SAVEDIR_SORT_NAME,
    //SAVEDIR_SORT_INODE
};

ARGMATCH_VERIFY (sort_mode_arg, sort_mode_flag);

static error_t
parse_opt (int key, char *arg, struct argp_state *state)
{
  struct tar_args *args = state->input;

  switch (key)
    {
    case ARGP_KEY_ARG:
      /* File name or non-parsed option, because of ARGP_IN_ORDER */
      name_add_name (arg, MAKE_INCL_OPTIONS (args));
      args->input_files = true;
      break;

    case 'A':
      set_subcommand_option (CAT_SUBCOMMAND);
      break;

    case 'a':
      args->compress_autodetect = true;
      break;

    case NO_AUTO_COMPRESS_OPTION:
      args->compress_autodetect = false;
      break;

    case 'b':
      {
	uintmax_t u;
	if (! (xstrtoumax (arg, 0, 10, &u, "") == LONGINT_OK
	       && u == (blocking_factor = u)
	       && 0 < blocking_factor
	       && u == (record_size = u * BLOCKSIZE) / BLOCKSIZE))
	  USAGE_ERROR ((0, 0, "%s: %s", quotearg_colon (arg),
			_("Invalid blocking factor")));
      }
      break;

    case 'B':
      /* Try to reblock input records.  For reading 4.2BSD pipes.  */

      /* It would surely make sense to exchange -B and -R, but it seems
	 that -B has been used for a long while in Sun tar and most
	 BSD-derived systems.  This is a consequence of the block/record
	 terminology confusion.  */

      read_full_records_option = true;
      break;

    case 'c':
      set_subcommand_option (CREATE_SUBCOMMAND);
      break;

    case 'C':
      name_add_dir (arg);
      break;

    case 'd':
      set_subcommand_option (DIFF_SUBCOMMAND);
      break;

    case 'f':
      if (archive_names == allocated_archive_names)
	archive_name_array = x2nrealloc (archive_name_array,
					 &allocated_archive_names,
					 sizeof (archive_name_array[0]));

      archive_name_array[archive_names++] = arg;
      break;

    case 'F':
      /* Since -F is only useful with -M, make it implied.  Run this
	 script at the end of each tape.  */

      info_script_option = arg;
      multi_volume_option = true;
      break;

    case FULL_TIME_OPTION:
      full_time_option = true;
      break;

    case 'g':
      listed_incremental_option = arg;
      after_date_option = true;
      /* Fall through.  */

    case 'G':
      /* We are making an incremental dump (FIXME: are we?); save
	 directories at the beginning of the archive, and include in each
	 directory its contents.  */

      incremental_option = true;
      break;

    case 'h':
      /* Follow symbolic links.  */
      dereference_option = true;
      break;

    case HARD_DEREFERENCE_OPTION:
      hard_dereference_option = true;
      break;

    case 'i':
      /* Ignore zero blocks (eofs).  This can't be the default,
	 because Unix tar writes two blocks of zeros, then pads out
	 the record with garbage.  */

      ignore_zeros_option = true;
      break;

    case 'j':
      set_use_compress_program_option (BZIP2_PROGRAM);
      break;

    case 'J':
      set_use_compress_program_option (XZ_PROGRAM);
      break;

    case 'k':
      /* Don't replace existing files.  */
      old_files_option = KEEP_OLD_FILES;
      break;

    case 'K':
      starting_file_option = true;
      addname (arg, 0, true, NULL);
      break;

    case ONE_FILE_SYSTEM_OPTION:
      /* When dumping directories, don't dump files/subdirectories
	 that are on other filesystems. */
      one_file_system_option = true;
      break;

    case ONE_TOP_LEVEL_OPTION:
      one_top_level_option = true;
      one_top_level_dir = arg;
      break;

    case 'l':
      check_links_option = 1;
      break;

    case 'L':
      {
	uintmax_t u;
	char *p;

	if (xstrtoumax (arg, &p, 10, &u, TAR_SIZE_SUFFIXES) != LONGINT_OK)
	  USAGE_ERROR ((0, 0, "%s: %s", quotearg_colon (arg),
			_("Invalid tape length")));
	if (p > arg && !strchr (TAR_SIZE_SUFFIXES, p[-1]))
	  tape_length_option = 1024 * (tarlong) u;
	else
	  tape_length_option = (tarlong) u;
	multi_volume_option = true;
      }
      break;

    case LEVEL_OPTION:
      {
	char *p;
	incremental_level = strtoul (arg, &p, 10);
	if (*p)
	  USAGE_ERROR ((0, 0, _("Invalid incremental level value")));
      }
      break;

    case LZIP_OPTION:
      set_use_compress_program_option (LZIP_PROGRAM);
      break;

    case LZMA_OPTION:
      set_use_compress_program_option (LZMA_PROGRAM);
      break;

    case LZOP_OPTION:
      set_use_compress_program_option (LZOP_PROGRAM);
      break;

    case 'm':
      touch_option = true;
      break;

    case 'M':
      /* Make multivolume archive: when we can't write any more into
	 the archive, re-open it, and continue writing.  */

      multi_volume_option = true;
      break;

    case MTIME_OPTION:
      get_date_or_file (args, "--mtime", arg, &mtime_option);
      set_mtime_option = true;
      break;

    case 'n':
      seek_option = 1;
      break;

    case NO_SEEK_OPTION:
      seek_option = 0;
      break;

    case 'N':
      after_date_option = true;
      /* Fall through.  */

    case NEWER_MTIME_OPTION:
      if (NEWER_OPTION_INITIALIZED (newer_mtime_option))
	USAGE_ERROR ((0, 0, _("More than one threshold date")));
      get_date_or_file (args,
			key == NEWER_MTIME_OPTION ? "--newer-mtime"
			: "--after-date", arg, &newer_mtime_option);
      break;

    case 'o':
      args->o_option = true;
      break;

    case 'O':
      to_stdout_option = true;
      break;

    case 'p':
      same_permissions_option = true;
      break;

    case 'P':
      absolute_names_option = true;
      break;

    case 'r':
      set_subcommand_option (APPEND_SUBCOMMAND);
      break;

    case 'R':
      /* Print block numbers for debugging bad tar archives.  */

      /* It would surely make sense to exchange -B and -R, but it seems
	 that -B has been used for a long while in Sun tar and most
	 BSD-derived systems.  This is a consequence of the block/record
	 terminology confusion.  */

      block_number_option = true;
      break;

    case 's':
      /* Names to extract are sorted.  */

      same_order_option = true;
      break;

    case 'S':
      sparse_option = true;
      break;

    case SKIP_OLD_FILES_OPTION:
      old_files_option = SKIP_OLD_FILES;
      break;

    case SPARSE_VERSION_OPTION:
      sparse_option = true;
      {
	char *p;
	tar_sparse_major = strtoul (arg, &p, 10);
	if (*p)
	  {
	    if (*p != '.')
	      USAGE_ERROR ((0, 0, _("Invalid sparse version value")));
	    tar_sparse_minor = strtoul (p + 1, &p, 10);
	    if (*p)
	      USAGE_ERROR ((0, 0, _("Invalid sparse version value")));
	  }
      }
      break;

    case 't':
      set_subcommand_option (LIST_SUBCOMMAND);
      verbose_option++;
      break;

    case TEST_LABEL_OPTION:
      set_subcommand_option (TEST_LABEL_SUBCOMMAND);
      break;

    case 'T':
      name_add_file (arg, filename_terminator);
      /* Indicate we've been given -T option. This is for backward
	 compatibility only, so that `tar cfT archive /dev/null will
	 succeed */
      files_from_option = true;
      break;

    case 'u':
      set_subcommand_option (UPDATE_SUBCOMMAND);
      break;

    case 'U':
      old_files_option = UNLINK_FIRST_OLD_FILES;
      break;

    case UTC_OPTION:
      utc_option = true;
      break;

    case 'v':
      verbose_option++;
      warning_option |= WARN_VERBOSE_WARNINGS;
      break;

    case 'V':
      volume_label_option = arg;
      break;

    case 'w':
      interactive_option = true;
      break;

    case 'W':
      verify_option = true;
      break;

    case 'x':
      set_subcommand_option (EXTRACT_SUBCOMMAND);
      break;

    case 'X':
      if (add_exclude_file (add_exclude, excluded, arg,
			    MAKE_EXCL_OPTIONS (args), '\n')
	  != 0)
	{
	  int e = errno;
	  FATAL_ERROR ((0, e, "%s", quotearg_colon (arg)));
	}
      break;

    case 'z':
      set_use_compress_program_option (GZIP_PROGRAM);
      break;

    case 'Z':
      set_use_compress_program_option (COMPRESS_PROGRAM);
      break;

    case ANCHORED_OPTION:
      args->matching_flags |= EXCLUDE_ANCHORED;
      break;

    case ATIME_PRESERVE_OPTION:
      atime_preserve_option =
	(arg
	 ? XARGMATCH ("--atime-preserve", arg,
		      atime_preserve_args, atime_preserve_types)
	 : replace_atime_preserve);
      if (! O_NOATIME && atime_preserve_option == system_atime_preserve)
	FATAL_ERROR ((0, 0,
		      _("--atime-preserve='system' is not supported"
			" on this platform")));
      break;

    case CHECK_DEVICE_OPTION:
      check_device_option = true;
      break;

    case NO_CHECK_DEVICE_OPTION:
      check_device_option = false;
      break;

    case CHECKPOINT_OPTION:
      if (arg)
	{
	  char *p;

	  if (*arg == '.')
	    {
	      checkpoint_compile_action (".");
	      arg++;
	    }
	  checkpoint_option = strtoul (arg, &p, 0);
	  if (*p)
	    FATAL_ERROR ((0, 0,
			  _("--checkpoint value is not an integer")));
	}
      else
	checkpoint_option = DEFAULT_CHECKPOINT;
      break;

    case CHECKPOINT_ACTION_OPTION:
      checkpoint_compile_action (arg);
      break;

    case BACKUP_OPTION:
      backup_option = true;
      if (arg)
	args->version_control_string = arg;
      break;

    case DELAY_DIRECTORY_RESTORE_OPTION:
      delay_directory_restore_option = true;
      break;

    case NO_DELAY_DIRECTORY_RESTORE_OPTION:
      delay_directory_restore_option = false;
      break;

    case DELETE_OPTION:
      set_subcommand_option (DELETE_SUBCOMMAND);
      break;

    case EXCLUDE_BACKUPS_OPTION:
      add_exclude_array (backup_file_table, EXCLUDE_WILDCARDS);
      break;

    case EXCLUDE_OPTION:
      add_exclude (excluded, arg, MAKE_EXCL_OPTIONS (args));
      break;

    case EXCLUDE_CACHES_OPTION:
      add_exclusion_tag ("CACHEDIR.TAG", exclusion_tag_contents,
			 cachedir_file_p);
      break;

    case EXCLUDE_CACHES_UNDER_OPTION:
      add_exclusion_tag ("CACHEDIR.TAG", exclusion_tag_under,
			 cachedir_file_p);
      break;

    case EXCLUDE_CACHES_ALL_OPTION:
      add_exclusion_tag ("CACHEDIR.TAG", exclusion_tag_all,
			 cachedir_file_p);
      break;

    case EXCLUDE_IGNORE_OPTION:
      excfile_add (arg, EXCL_NON_RECURSIVE);
      break;

    case EXCLUDE_IGNORE_RECURSIVE_OPTION:
      excfile_add (arg, EXCL_RECURSIVE);
      break;

    case EXCLUDE_TAG_OPTION:
      add_exclusion_tag (arg, exclusion_tag_contents, NULL);
      break;

    case EXCLUDE_TAG_UNDER_OPTION:
      add_exclusion_tag (arg, exclusion_tag_under, NULL);
      break;

    case EXCLUDE_TAG_ALL_OPTION:
      add_exclusion_tag (arg, exclusion_tag_all, NULL);
      break;

    case EXCLUDE_VCS_OPTION:
      add_exclude_array (vcs_file_table, 0);
      break;

    case EXCLUDE_VCS_IGNORES_OPTION:
      exclude_vcs_ignores ();
      break;

    case FORCE_LOCAL_OPTION:
      force_local_option = true;
      break;

    case 'H':
      set_archive_format (arg);
      break;

    case INDEX_FILE_OPTION:
      index_file_name = arg;
      break;

    case IGNORE_CASE_OPTION:
      args->matching_flags |= FNM_CASEFOLD;
      break;

    case IGNORE_COMMAND_ERROR_OPTION:
      ignore_command_error_option = true;
      break;

    case IGNORE_FAILED_READ_OPTION:
      ignore_failed_read_option = true;
      break;

    case KEEP_DIRECTORY_SYMLINK_OPTION:
      keep_directory_symlink_option = true;
      break;

    case KEEP_NEWER_FILES_OPTION:
      old_files_option = KEEP_NEWER_FILES;
      break;

    case GROUP_OPTION:
      {
	uintmax_t u = parse_owner_group (arg, TYPE_MAXIMUM (gid_t),
					 &group_name_option);
	if (u == UINTMAX_MAX)
	  {
	    group_option = -1;
	    if (group_name_option)
	      gname_to_gid (group_name_option, &group_option);
	  }
	else
	  group_option = u;
      }
      break;

    case MODE_OPTION:
      mode_option = mode_compile (arg);
      if (!mode_option)
	FATAL_ERROR ((0, 0, _("Invalid mode given on option")));
      initial_umask = umask (0);
      umask (initial_umask);
      break;

    case NO_ANCHORED_OPTION:
      args->include_anchored = 0; /* Clear the default for comman line args */
      args->matching_flags &= ~ EXCLUDE_ANCHORED;
      break;

    case NO_IGNORE_CASE_OPTION:
      args->matching_flags &= ~ FNM_CASEFOLD;
      break;

    case NO_IGNORE_COMMAND_ERROR_OPTION:
      ignore_command_error_option = false;
      break;

    case NO_OVERWRITE_DIR_OPTION:
      old_files_option = NO_OVERWRITE_DIR_OLD_FILES;
      break;

    case NO_QUOTE_CHARS_OPTION:
      for (;*arg; arg++)
	set_char_quoting (NULL, *arg, 0);
      break;

    case NO_WILDCARDS_OPTION:
      args->wildcards = disable_wildcards;
      break;

    case NO_WILDCARDS_MATCH_SLASH_OPTION:
      args->matching_flags |= FNM_FILE_NAME;
      break;

    case NULL_OPTION:
      filename_terminator = '\0';
      break;

    case NO_NULL_OPTION:
      filename_terminator = '\n';
      break;

    case NUMERIC_OWNER_OPTION:
      numeric_owner_option = true;
      break;

    case OCCURRENCE_OPTION:
      if (!arg)
	occurrence_option = 1;
      else
	{
	  uintmax_t u;
	  if (xstrtoumax (arg, 0, 10, &u, "") == LONGINT_OK)
	    occurrence_option = u;
	  else
	    FATAL_ERROR ((0, 0, "%s: %s", quotearg_colon (arg),
			  _("Invalid number")));
	}
      break;

    case OLD_ARCHIVE_OPTION:
      set_archive_format ("v7");
      break;

    case OVERWRITE_DIR_OPTION:
      old_files_option = DEFAULT_OLD_FILES;
      break;

    case OVERWRITE_OPTION:
      old_files_option = OVERWRITE_OLD_FILES;
      break;

    case OWNER_OPTION:
      {
	uintmax_t u = parse_owner_group (arg, TYPE_MAXIMUM (uid_t),
					 &owner_name_option);
	if (u == UINTMAX_MAX)
	  {
	    owner_option = -1;
	    if (owner_name_option)
	      uname_to_uid (owner_name_option, &owner_option);
	  }
	else
	  owner_option = u;
      }
      break;

    case QUOTE_CHARS_OPTION:
      for (;*arg; arg++)
	set_char_quoting (NULL, *arg, 1);
      break;

    case QUOTING_STYLE_OPTION:
      tar_set_quoting_style (arg);
      break;

    case PAX_OPTION:
      {
	char *tmp = expand_pax_option (args, arg);
	args->pax_option = true;
	xheader_set_option (tmp);
	free (tmp);
      }
      break;

    case POSIX_OPTION:
      set_archive_format ("posix");
      break;

    case PRESERVE_OPTION:
      /* FIXME: What it is good for? */
      same_permissions_option = true;
      same_order_option = true;
      WARN ((0, 0, _("The --preserve option is deprecated, "
		     "use --preserve-permissions --preserve-order instead")));
      break;

    case RECORD_SIZE_OPTION:
      {
	uintmax_t u;

	if (! (xstrtoumax (arg, NULL, 10, &u, TAR_SIZE_SUFFIXES) == LONGINT_OK
	       && u == (size_t) u))
	  USAGE_ERROR ((0, 0, "%s: %s", quotearg_colon (arg),
			_("Invalid record size")));
	record_size = u;
	if (record_size % BLOCKSIZE != 0)
	  USAGE_ERROR ((0, 0, _("Record size must be a multiple of %d."),
			BLOCKSIZE));
	blocking_factor = record_size / BLOCKSIZE;
      }
      break;

    case RECURSIVE_UNLINK_OPTION:
      recursive_unlink_option = true;
      break;

    case REMOVE_FILES_OPTION:
      remove_files_option = true;
      break;

    case RESTRICT_OPTION:
      restrict_option = true;
      break;

    case RMT_COMMAND_OPTION:
      rmt_command = arg;
      break;

    case RSH_COMMAND_OPTION:
      rsh_command_option = arg;
      break;

    case SHOW_DEFAULTS_OPTION:
      {
	char *s = format_default_settings ();
	printf ("%s\n", s);
	close_stdout ();
	free (s);
	exit (0);
      }

    case SHOW_SNAPSHOT_FIELD_RANGES_OPTION:
      show_snapshot_field_ranges ();
      close_stdout ();
      exit (0);

    case STRIP_COMPONENTS_OPTION:
      {
	uintmax_t u;
	if (! (xstrtoumax (arg, 0, 10, &u, "") == LONGINT_OK
	       && u == (size_t) u))
	  USAGE_ERROR ((0, 0, "%s: %s", quotearg_colon (arg),
			_("Invalid number of elements")));
	strip_name_components = u;
      }
      break;

    case SHOW_OMITTED_DIRS_OPTION:
      show_omitted_dirs_option = true;
      break;

    case SHOW_TRANSFORMED_NAMES_OPTION:
      show_transformed_names_option = true;
      break;

    case SORT_OPTION:
      savedir_sort_order = XARGMATCH ("--sort", arg,
				      sort_mode_arg, sort_mode_flag);
      break;

    case SUFFIX_OPTION:
      backup_option = true;
      args->backup_suffix_string = arg;
      break;

    case TO_COMMAND_OPTION:
      if (to_command_option)
        USAGE_ERROR ((0, 0, _("Only one --to-command option allowed")));
      to_command_option = arg;
      break;

    case TOTALS_OPTION:
      if (arg)
	set_stat_signal (arg);
      else
	totals_option = true;
      break;

    case TRANSFORM_OPTION:
      set_transform_expr (arg);
      break;

    case 'I':
      set_use_compress_program_option (arg);
      break;

    case VOLNO_FILE_OPTION:
      volno_file_option = arg;
      break;

    case WILDCARDS_OPTION:
      args->wildcards = enable_wildcards;
      break;

    case WILDCARDS_MATCH_SLASH_OPTION:
      args->matching_flags &= ~ FNM_FILE_NAME;
      break;

    case NO_RECURSION_OPTION:
      recursion_option = 0;
      break;

    case NO_SAME_OWNER_OPTION:
      same_owner_option = -1;
      break;

    case NO_SAME_PERMISSIONS_OPTION:
      same_permissions_option = -1;
      break;

    case ACLS_OPTION:
      set_archive_format ("posix");
      acls_option = 1;
      break;

    case NO_ACLS_OPTION:
      acls_option = -1;
      break;

    case SELINUX_CONTEXT_OPTION:
      set_archive_format ("posix");
      selinux_context_option = 1;
      break;

    case NO_SELINUX_CONTEXT_OPTION:
      selinux_context_option = -1;
      break;

    case XATTR_OPTION:
      set_xattr_option (1);
      break;

    case NO_XATTR_OPTION:
      set_xattr_option (-1);
      break;

    case XATTR_INCLUDE:
    case XATTR_EXCLUDE:
      set_xattr_option (1);
      xattrs_mask_add (arg, (key == XATTR_INCLUDE));
      break;

    case RECURSION_OPTION:
      recursion_option = FNM_LEADING_DIR;
      break;

    case SAME_OWNER_OPTION:
      same_owner_option = 1;
      break;

    case UNQUOTE_OPTION:
      unquote_option = true;
      break;

    case NO_UNQUOTE_OPTION:
      unquote_option = false;
      break;

    case WARNING_OPTION:
      set_warning_option (arg);
      break;

    case '0':
    case '1':
    case '2':
    case '3':
    case '4':
    case '5':
    case '6':
    case '7':

#ifdef DEVICE_PREFIX
      {
	int device = key - '0';
	int density;
	static char buf[sizeof DEVICE_PREFIX + 10];
	char *cursor;

	if (arg[1])
	  argp_error (state, _("Malformed density argument: %s"), quote (arg));

	strcpy (buf, DEVICE_PREFIX);
	cursor = buf + strlen (buf);

#ifdef DENSITY_LETTER

	sprintf (cursor, "%d%c", device, arg[0]);

#else /* not DENSITY_LETTER */

	switch (arg[0])
	  {
	  case 'l':
	    device += LOW_DENSITY_NUM;
	    break;

	  case 'm':
	    device += MID_DENSITY_NUM;
	    break;

	  case 'h':
	    device += HIGH_DENSITY_NUM;
	    break;

	  default:
	    argp_error (state, _("Unknown density: '%c'"), arg[0]);
	  }
	sprintf (cursor, "%d", device);

#endif /* not DENSITY_LETTER */

	if (archive_names == allocated_archive_names)
	  archive_name_array = x2nrealloc (archive_name_array,
					   &allocated_archive_names,
					   sizeof (archive_name_array[0]));
	archive_name_array[archive_names++] = xstrdup (buf);
      }
      break;

#else /* not DEVICE_PREFIX */

      argp_error (state,
		  _("Options '-[0-7][lmh]' not supported by *this* tar"));

#endif /* not DEVICE_PREFIX */

    default:
      return ARGP_ERR_UNKNOWN;
    }
  return 0;
}

static struct argp argp = {
  options,
  parse_opt,
  N_("[FILE]..."),
  doc,
  NULL,
  tar_help_filter,
  NULL
};

void
usage (int status)
{
  argp_help (&argp, stderr, ARGP_HELP_SEE, (char*) program_name);
  close_stdout ();
  exit (status);
}

/* Parse the options for tar.  */

static struct argp_option *
find_argp_option (struct argp_option *o, int letter)
{
  for (;
       !(o->name == NULL
	 && o->key == 0
	 && o->arg == 0
	 && o->flags == 0
	 && o->doc == NULL); o++)
    if (o->key == letter)
      return o;
  return NULL;
}

static const char *tar_authors[] = {
  "John Gilmore",
  "Jay Fenlason",
  NULL
};

/* Subcommand classes */
#define SUBCL_READ    0x01   /* subcommand reads from the archive */
#define SUBCL_WRITE   0x02   /* subcommand writes to the archive */
#define SUBCL_UPDATE  0x04   /* subcommand updates existing archive */
#define SUBCL_TEST    0x08   /* subcommand tests archive header or meta-info */
#define SUBCL_OCCUR   0x10   /* subcommand allows the use of the occurrence
				option */

static int subcommand_class[] = {
  /* UNKNOWN_SUBCOMMAND */     0,
  /* APPEND_SUBCOMMAND  */     SUBCL_WRITE|SUBCL_UPDATE,
  /* CAT_SUBCOMMAND     */     SUBCL_WRITE,
  /* CREATE_SUBCOMMAND  */     SUBCL_WRITE,
  /* DELETE_SUBCOMMAND  */     SUBCL_WRITE|SUBCL_UPDATE|SUBCL_OCCUR,
  /* DIFF_SUBCOMMAND    */     SUBCL_READ|SUBCL_OCCUR,
  /* EXTRACT_SUBCOMMAND */     SUBCL_READ|SUBCL_OCCUR,
  /* LIST_SUBCOMMAND    */     SUBCL_READ|SUBCL_OCCUR,
  /* UPDATE_SUBCOMMAND  */     SUBCL_WRITE|SUBCL_UPDATE,
  /* TEST_LABEL_SUBCOMMAND */  SUBCL_TEST
};

/* Return t if the subcommand_option is in class(es) f */
#define IS_SUBCOMMAND_CLASS(f) (subcommand_class[subcommand_option] & (f))

static struct tar_args args;

static void
option_conflict_error (const char *a, const char *b)
{
  /* TRANSLATORS: Both %s in this statement are replaced with
     option names. */
  USAGE_ERROR ((0, 0, _("'%s' cannot be used with '%s'"), a, b));
}

static void
decode_options (int argc, char **argv)
{
  int idx;

  argp_version_setup ("tar", tar_authors);

  /* Set some default option values.  */
  args.textual_date = NULL;
  args.wildcards = default_wildcards;
  args.matching_flags = 0;
  args.include_anchored = EXCLUDE_ANCHORED;
  args.o_option = false;
  args.pax_option = false;
  args.backup_suffix_string = getenv ("SIMPLE_BACKUP_SUFFIX");
  args.version_control_string = 0;
  args.input_files = false;
  args.compress_autodetect = false;

  subcommand_option = UNKNOWN_SUBCOMMAND;
  archive_format = DEFAULT_FORMAT;
  blocking_factor = DEFAULT_BLOCKING;
  record_size = DEFAULT_BLOCKING * BLOCKSIZE;
  excluded = new_exclude ();

  newer_mtime_option.tv_sec = TYPE_MINIMUM (time_t);
  newer_mtime_option.tv_nsec = -1;
  recursion_option = FNM_LEADING_DIR;
  unquote_option = true;
  tar_sparse_major = 1;
  tar_sparse_minor = 0;

  savedir_sort_order = SAVEDIR_SORT_NONE;

  owner_option = -1; owner_name_option = NULL;
  group_option = -1; group_name_option = NULL;

  check_device_option = true;

  incremental_level = -1;

  seek_option = -1;

  /* Convert old-style tar call by exploding option element and rearranging
     options accordingly.  */

  if (argc > 1 && argv[1][0] != '-')
    {
      int new_argc;		/* argc value for rearranged arguments */
      char **new_argv;		/* argv value for rearranged arguments */
      char *const *in;		/* cursor into original argv */
      char **out;		/* cursor into rearranged argv */
      const char *letter;	/* cursor into old option letters */
      char buffer[3];		/* constructed option buffer */

      /* Initialize a constructed option.  */

      buffer[0] = '-';
      buffer[2] = '\0';

      /* Allocate a new argument array, and copy program name in it.  */

      new_argc = argc - 1 + strlen (argv[1]);
      new_argv = xmalloc ((new_argc + 1) * sizeof (char *));
      in = argv;
      out = new_argv;
      *out++ = *in++;

      /* Copy each old letter option as a separate option, and have the
	 corresponding argument moved next to it.  */

      for (letter = *in++; *letter; letter++)
	{
	  struct argp_option *opt;

	  buffer[1] = *letter;
	  *out++ = xstrdup (buffer);
	  opt = find_argp_option (options, *letter);
	  if (opt && opt->arg)
	    {
	      if (in < argv + argc)
		*out++ = *in++;
	      else
		USAGE_ERROR ((0, 0, _("Old option '%c' requires an argument."),
			      *letter));
	    }
	}

      /* Copy all remaining options.  */

      while (in < argv + argc)
	*out++ = *in++;
      *out = 0;

      /* Replace the old option list by the new one.  */

      argc = new_argc;
      argv = new_argv;
    }

  /* Parse all options and non-options as they appear.  */

  prepend_default_options (getenv ("TAR_OPTIONS"), &argc, &argv);

  if (argp_parse (&argp, argc, argv, ARGP_IN_ORDER, &idx, &args))
    exit (TAREXIT_FAILURE);

  /* Special handling for 'o' option:

     GNU tar used to say "output old format".
     UNIX98 tar says don't chown files after extracting (we use
     "--no-same-owner" for this).

     The old GNU tar semantics is retained when used with --create
     option, otherwise UNIX98 semantics is assumed */

  if (args.o_option)
    {
      if (subcommand_option == CREATE_SUBCOMMAND)
	{
	  /* GNU Tar <= 1.13 compatibility */
	  set_archive_format ("v7");
	}
      else
	{
	  /* UNIX98 compatibility */
	  same_owner_option = -1;
	}
    }

  /* Handle operands after any "--" argument.  */
  for (; idx < argc; idx++)
    {
      name_add_name (argv[idx], MAKE_INCL_OPTIONS (&args));
      args.input_files = true;
    }

  /* Warn about implicit use of the wildcards in command line arguments.
     See TODO */
  warn_regex_usage = args.wildcards == default_wildcards;

  /* Derive option values and check option consistency.  */

  if (archive_format == DEFAULT_FORMAT)
    {
      if (args.pax_option)
	archive_format = POSIX_FORMAT;
      else
	archive_format = DEFAULT_ARCHIVE_FORMAT;
    }

  if ((volume_label_option && subcommand_option == CREATE_SUBCOMMAND)
      || incremental_option
      || multi_volume_option
      || sparse_option)
    assert_format (FORMAT_MASK (OLDGNU_FORMAT)
		   | FORMAT_MASK (GNU_FORMAT)
		   | FORMAT_MASK (POSIX_FORMAT));

  if (occurrence_option)
    {
      if (!args.input_files)
	USAGE_ERROR ((0, 0,
		      _("--occurrence is meaningless without a file list")));
      if (!IS_SUBCOMMAND_CLASS (SUBCL_OCCUR))
	option_conflict_error ("--occurrence",
			       subcommand_string (subcommand_option));
    }

  if (archive_names == 0)
    {
      /* If no archive file name given, try TAPE from the environment, or
	 else, DEFAULT_ARCHIVE from the configuration process.  */

      archive_names = 1;
      archive_name_array[0] = getenv ("TAPE");
      if (! archive_name_array[0])
	archive_name_array[0] = DEFAULT_ARCHIVE;
    }

  /* Allow multiple archives only with '-M'.  */

  if (archive_names > 1 && !multi_volume_option)
    USAGE_ERROR ((0, 0,
		  _("Multiple archive files require '-M' option")));

  if (listed_incremental_option
      && NEWER_OPTION_INITIALIZED (newer_mtime_option))
    option_conflict_error ("--listed-incremental", "--newer");

  if (incremental_level != -1 && !listed_incremental_option)
    WARN ((0, 0,
	   _("--level is meaningless without --listed-incremental")));

  if (volume_label_option)
    {
      if (archive_format == GNU_FORMAT || archive_format == OLDGNU_FORMAT)
	{
	  size_t volume_label_max_len =
	    (sizeof current_header->header.name
	     - 1 /* for trailing '\0' */
	     - (multi_volume_option
		? (sizeof " Volume "
		   - 1 /* for null at end of " Volume " */
		   + INT_STRLEN_BOUND (int) /* for volume number */
		   - 1 /* for sign, as 0 <= volno */)
		: 0));
	  if (volume_label_max_len < strlen (volume_label_option))
	    USAGE_ERROR ((0, 0,
			  ngettext ("%s: Volume label is too long (limit is %lu byte)",
				    "%s: Volume label is too long (limit is %lu bytes)",
				    volume_label_max_len),
			  quotearg_colon (volume_label_option),
			  (unsigned long) volume_label_max_len));
	}
      /* else FIXME
	 Label length in PAX format is limited by the volume size. */
    }

  if (verify_option)
    {
      if (multi_volume_option)
	USAGE_ERROR ((0, 0, _("Cannot verify multi-volume archives")));
      if (use_compress_program_option)
	USAGE_ERROR ((0, 0, _("Cannot verify compressed archives")));
      if (!IS_SUBCOMMAND_CLASS (SUBCL_WRITE))
	option_conflict_error ("--verify",
			       subcommand_string (subcommand_option));
    }

  if (use_compress_program_option)
    {
      if (multi_volume_option)
	USAGE_ERROR ((0, 0, _("Cannot use multi-volume compressed archives")));
      if (IS_SUBCOMMAND_CLASS (SUBCL_UPDATE))
	USAGE_ERROR ((0, 0, _("Cannot update compressed archives")));
      if (subcommand_option == CAT_SUBCOMMAND)
	USAGE_ERROR ((0, 0, _("Cannot concatenate compressed archives")));
    }

  /* It is no harm to use --pax-option on non-pax archives in archive
     reading mode. It may even be useful, since it allows to override
     file attributes from tar headers. Therefore I allow such usage.
     --gray */
  if (args.pax_option
      && archive_format != POSIX_FORMAT
      && !IS_SUBCOMMAND_CLASS (SUBCL_READ))
    USAGE_ERROR ((0, 0, _("--pax-option can be used only on POSIX archives")));

  /* star creates non-POSIX typed archives with xattr support, so allow the
     extra headers when reading */
  if ((acls_option > 0)
      && archive_format != POSIX_FORMAT
      && !IS_SUBCOMMAND_CLASS (SUBCL_READ))
    USAGE_ERROR ((0, 0, _("--acls can be used only on POSIX archives")));

  if ((selinux_context_option > 0)
      && archive_format != POSIX_FORMAT
      && !IS_SUBCOMMAND_CLASS (SUBCL_READ))
    USAGE_ERROR ((0, 0, _("--selinux can be used only on POSIX archives")));

  if ((xattrs_option > 0)
      && archive_format != POSIX_FORMAT
      && !IS_SUBCOMMAND_CLASS (SUBCL_READ))
    USAGE_ERROR ((0, 0, _("--xattrs can be used only on POSIX archives")));

  if (starting_file_option && !IS_SUBCOMMAND_CLASS (SUBCL_READ))
    option_conflict_error ("--starting-file",
			   subcommand_string (subcommand_option));

  if (same_order_option && !IS_SUBCOMMAND_CLASS (SUBCL_READ))
    option_conflict_error ("--same-order",
			   subcommand_string (subcommand_option));

  if (one_top_level_option)
    {
      char *base;

      if (absolute_names_option)
	option_conflict_error ("--one-top-level", "--absolute-names");

      if (!one_top_level_dir)
	{
	  /* If the user wants to guarantee that everything is under one
	     directory, determine its name now and let it be created later.  */
	  base = base_name (archive_name_array[0]);
	  one_top_level_dir = strip_compression_suffix (base);
	  free (base);

	  if (!one_top_level_dir)
	    USAGE_ERROR ((0, 0,
			  _("Cannot deduce top-level directory name; "
			    "please set it explicitly with --one-top-level=DIR")));
	}
    }

  /* If ready to unlink hierarchies, so we are for simpler files.  */
  if (recursive_unlink_option)
    old_files_option = UNLINK_FIRST_OLD_FILES;

  /* Flags for accessing files to be read from or copied into.  POSIX says
     O_NONBLOCK has unspecified effect on most types of files, but in
     practice it never harms and sometimes helps.  */
  {
    int base_open_flags =
      (O_BINARY | O_CLOEXEC | O_NOCTTY | O_NONBLOCK
       | (dereference_option ? 0 : O_NOFOLLOW)
       | (atime_preserve_option == system_atime_preserve ? O_NOATIME : 0));
    open_read_flags = O_RDONLY | base_open_flags;
    open_searchdir_flags = O_SEARCH | O_DIRECTORY | base_open_flags;
  }
  fstatat_flags = dereference_option ? 0 : AT_SYMLINK_NOFOLLOW;

  if (subcommand_option == TEST_LABEL_SUBCOMMAND)
    {
      /* --test-label is silent if the user has specified the label name to
	 compare against. */
      if (!args.input_files)
	verbose_option++;
    }
  else if (utc_option)
    verbose_option = 2;

  if (tape_length_option && tape_length_option < record_size)
    USAGE_ERROR ((0, 0, _("Volume length cannot be less than record size")));

  if (same_order_option && listed_incremental_option)
    option_conflict_error ("--preserve-order", "--listed-incremental");

  /* Forbid using -c with no input files whatsoever.  Check that '-f -',
     explicit or implied, is used correctly.  */

  switch (subcommand_option)
    {
    case CREATE_SUBCOMMAND:
      if (!args.input_files && !files_from_option)
	USAGE_ERROR ((0, 0,
		      _("Cowardly refusing to create an empty archive")));
      if (args.compress_autodetect && archive_names
	  && strcmp (archive_name_array[0], "-"))
	set_compression_program_by_suffix (archive_name_array[0],
					   use_compress_program_option);
      break;

    case EXTRACT_SUBCOMMAND:
    case LIST_SUBCOMMAND:
    case DIFF_SUBCOMMAND:
    case TEST_LABEL_SUBCOMMAND:
      for (archive_name_cursor = archive_name_array;
	   archive_name_cursor < archive_name_array + archive_names;
	   archive_name_cursor++)
	if (!strcmp (*archive_name_cursor, "-"))
	  request_stdin ("-f");
      break;

    case CAT_SUBCOMMAND:
    case UPDATE_SUBCOMMAND:
    case APPEND_SUBCOMMAND:
      for (archive_name_cursor = archive_name_array;
	   archive_name_cursor < archive_name_array + archive_names;
	   archive_name_cursor++)
	if (!strcmp (*archive_name_cursor, "-"))
	  USAGE_ERROR ((0, 0,
			_("Options '-Aru' are incompatible with '-f -'")));

    default:
      break;
    }

  /* Initialize stdlis */
  if (index_file_name)
    {
      stdlis = fopen (index_file_name, "w");
      if (! stdlis)
	open_fatal (index_file_name);
    }
  else
    stdlis = to_stdout_option ? stderr : stdout;

  archive_name_cursor = archive_name_array;

  /* Prepare for generating backup names.  */

  if (args.backup_suffix_string)
    simple_backup_suffix = xstrdup (args.backup_suffix_string);

  if (backup_option)
    {
      backup_type = xget_version ("--backup", args.version_control_string);
      /* No backup is needed either if explicitely disabled or if
	 the extracted files are not being written to disk. */
      if (backup_type == no_backups || EXTRACT_OVER_PIPE)
	backup_option = false;
    }

  checkpoint_finish_compile ();

  report_textual_dates (&args);
}

void
more_options (int argc, char **argv)
{
  int idx;
  if (argp_parse (&argp, argc, argv, ARGP_IN_ORDER,
		  &idx, &args))
    exit (TAREXIT_FAILURE);
}

/* Tar proper.  */

/* Main routine for tar.  */
int
main (int argc, char **argv)
{
  set_start_time ();
  set_program_name (argv[0]);

  setlocale (LC_ALL, "");
  bindtextdomain (PACKAGE, LOCALEDIR);
  textdomain (PACKAGE);

  exit_failure = TAREXIT_FAILURE;
  exit_status = TAREXIT_SUCCESS;
  error_hook = checkpoint_flush_actions;

  filename_terminator = '\n';
  set_quoting_style (0, DEFAULT_QUOTING_STYLE);

  /* Make sure we have first three descriptors available */
  stdopen ();

  /* Pre-allocate a few structures.  */

  allocated_archive_names = 10;
  archive_name_array =
    xmalloc (sizeof (const char *) * allocated_archive_names);
  archive_names = 0;

  /* System V fork+wait does not work if SIGCHLD is ignored.  */
  signal (SIGCHLD, SIG_DFL);

  /* Try to disable the ability to unlink a directory.  */
  priv_set_remove_linkdir ();

  /* Decode options.  */

  decode_options (argc, argv);

  name_init ();

  /* Main command execution.  */

  if (volno_file_option)
    init_volume_number ();

  switch (subcommand_option)
    {
    case UNKNOWN_SUBCOMMAND:
      USAGE_ERROR ((0, 0,
		    _("You must specify one of the '-Acdtrux', '--delete' or '--test-label' options")));

    case CAT_SUBCOMMAND:
    case UPDATE_SUBCOMMAND:
    case APPEND_SUBCOMMAND:
      update_archive ();
      break;

    case DELETE_SUBCOMMAND:
      delete_archive_members ();
      break;

    case CREATE_SUBCOMMAND:
      create_archive ();
      break;

    case EXTRACT_SUBCOMMAND:
      extr_init ();
      read_and (extract_archive);

      /* FIXME: should extract_finish () even if an ordinary signal is
	 received.  */
      extract_finish ();

      break;

    case LIST_SUBCOMMAND:
      read_and (list_archive);
      break;

    case DIFF_SUBCOMMAND:
      diff_init ();
      read_and (diff_archive);
      break;

    case TEST_LABEL_SUBCOMMAND:
      test_archive_label ();
    }

  checkpoint_finish ();

  if (totals_option)
    print_total_stats ();

  if (check_links_option)
    check_links ();

  if (volno_file_option)
    closeout_volume_number ();

  /* Dispose of allocated memory, and return.  */

  free (archive_name_array);
  xattrs_clear_setup ();
  name_term ();

  if (exit_status == TAREXIT_FAILURE)
    error (0, 0, _("Exiting with failure status due to previous errors"));

  if (stdlis == stdout)
    close_stdout ();
  else if (ferror (stderr) || fclose (stderr) != 0)
    set_exit_status (TAREXIT_FAILURE);

  return exit_status;
}

void
tar_stat_init (struct tar_stat_info *st)
{
  memset (st, 0, sizeof (*st));
}

/* Close the stream or file descriptor associated with ST, and remove
   all traces of it from ST.  Return true if successful, false (with a
   diagnostic) otherwise.  */
bool
tar_stat_close (struct tar_stat_info *st)
{
  int status = (st->dirstream ? closedir (st->dirstream)
		: 0 < st->fd ? close (st->fd)
		: 0);
  st->dirstream = 0;
  st->fd = 0;

  if (status == 0)
    return true;
  else
    {
      close_diag (st->orig_file_name);
      return false;
    }
}

void
tar_stat_destroy (struct tar_stat_info *st)
{
  tar_stat_close (st);
  xheader_xattr_free (st->xattr_map, st->xattr_map_size);
  free (st->orig_file_name);
  free (st->file_name);
  free (st->link_name);
  free (st->uname);
  free (st->gname);
  free (st->cntx_name);
  free (st->acls_a_ptr);
  free (st->acls_d_ptr);
  free (st->sparse_map);
  free (st->dumpdir);
  xheader_destroy (&st->xhdr);
  info_free_exclist (st);
  memset (st, 0, sizeof (*st));
}

/* Format mask for all available formats that support nanosecond
   timestamp resolution. */
#define NS_PRECISION_FORMAT_MASK FORMAT_MASK (POSIX_FORMAT)

/* Same as timespec_cmp, but ignore nanoseconds if current archive
   format does not provide sufficient resolution.  */
int
tar_timespec_cmp (struct timespec a, struct timespec b)
{
  if (!(FORMAT_MASK (current_format) & NS_PRECISION_FORMAT_MASK))
    a.tv_nsec = b.tv_nsec = 0;
  return timespec_cmp (a, b);
}

/* Set tar exit status to VAL, unless it is already indicating
   a more serious condition. This relies on the fact that the
   values of TAREXIT_ constants are ranged by severity. */
void
set_exit_status (int val)
{
  if (val > exit_status)
    exit_status = val;
}
