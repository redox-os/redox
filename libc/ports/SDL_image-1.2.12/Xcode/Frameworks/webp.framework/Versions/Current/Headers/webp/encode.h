// Copyright 2011 Google Inc.
//
// This code is licensed under the same terms as WebM:
//  Software License Agreement:  http://www.webmproject.org/license/software/
//  Additional IP Rights Grant:  http://www.webmproject.org/license/additional/
// -----------------------------------------------------------------------------
//
//   WebP encoder: main interface
//
// Author: Skal (pascal.massimino@gmail.com)

#ifndef WEBP_WEBP_ENCODE_H_
#define WEBP_WEBP_ENCODE_H_

#include <stdlib.h>

#include "./types.h"

#if defined(__cplusplus) || defined(c_plusplus)
extern "C" {
#endif

#define WEBP_ENCODER_ABI_VERSION 0x0003

// Return the encoder's version number, packed in hexadecimal using 8bits for
// each of major/minor/revision. E.g: v2.5.7 is 0x020507.
WEBP_EXTERN(int) WebPGetEncoderVersion(void);

//------------------------------------------------------------------------------
// One-stop-shop call! No questions asked:

// Returns the size of the compressed data (pointed to by *output), or 0 if
// an error occurred. The compressed data must be released by the caller
// using the call 'free(*output)'.
WEBP_EXTERN(size_t) WebPEncodeRGB(const uint8_t* rgb,
                                  int width, int height, int stride,
                                  float quality_factor, uint8_t** output);
WEBP_EXTERN(size_t) WebPEncodeBGR(const uint8_t* bgr,
                                  int width, int height, int stride,
                                  float quality_factor, uint8_t** output);
WEBP_EXTERN(size_t) WebPEncodeRGBA(const uint8_t* rgba,
                                   int width, int height, int stride,
                                   float quality_factor, uint8_t** output);
WEBP_EXTERN(size_t) WebPEncodeBGRA(const uint8_t* bgra,
                                   int width, int height, int stride,
                                   float quality_factor, uint8_t** output);

//------------------------------------------------------------------------------
// Coding parameters

typedef struct {
  float quality;         // between 0 (smallest file) and 100 (biggest)
  int target_size;       // if non-zero, set the desired target size in bytes.
                         // Takes precedence over the 'compression' parameter.
  float target_PSNR;     // if non-zero, specifies the minimal distortion to
                         // try to achieve. Takes precedence over target_size.
  int method;            // quality/speed trade-off (0=fast, 6=slower-better)
  int segments;          // maximum number of segments to use, in [1..4]
  int sns_strength;      // Spatial Noise Shaping. 0=off, 100=maximum.
  int filter_strength;   // range: [0 = off .. 100 = strongest]
  int filter_sharpness;  // range: [0 = off .. 7 = least sharp]
  int filter_type;       // filtering type: 0 = simple, 1 = strong
                         // (only used if filter_strength > 0 or autofilter > 0)
  int autofilter;        // Auto adjust filter's strength [0 = off, 1 = on]
  int pass;              // number of entropy-analysis passes (in [1..10]).

  int show_compressed;   // if true, export the compressed picture back.
                         // In-loop filtering is not applied.
  int preprocessing;     // preprocessing filter (0=none, 1=segment-smooth)
  int partitions;        // log2(number of token partitions) in [0..3]
                         // Default is set to 0 for easier progressive decoding.
  int partition_limit;   // quality degradation allowed to fit the 512k limit on
                         // prediction modes coding (0=no degradation, 100=full)
  int alpha_compression;  // Algorithm for encoding the alpha plane (0 = none,
                          // 1 = backward reference counts encoded with
                          // arithmetic encoder). Default is 1.
  int alpha_quality;      // Between 0 (smallest size) and 100 (lossless).
                          // Default is 100.
} WebPConfig;

// Enumerate some predefined settings for WebPConfig, depending on the type
// of source picture. These presets are used when calling WebPConfigPreset().
typedef enum {
  WEBP_PRESET_DEFAULT = 0,  // default preset.
  WEBP_PRESET_PICTURE,      // digital picture, like portrait, inner shot
  WEBP_PRESET_PHOTO,        // outdoor photograph, with natural lighting
  WEBP_PRESET_DRAWING,      // hand or line drawing, with high-contrast details
  WEBP_PRESET_ICON,         // small-sized colorful images
  WEBP_PRESET_TEXT          // text-like
} WebPPreset;

// Internal, version-checked, entry point
WEBP_EXTERN(int) WebPConfigInitInternal(
    WebPConfig* const, WebPPreset, float, int);

// Should always be called, to initialize a fresh WebPConfig structure before
// modification. Returns 0 in case of version mismatch. WebPConfigInit() must
// have succeeded before using the 'config' object.
static WEBP_INLINE int WebPConfigInit(WebPConfig* const config) {
  return WebPConfigInitInternal(config, WEBP_PRESET_DEFAULT, 75.f,
                                WEBP_ENCODER_ABI_VERSION);
}

// This function will initialize the configuration according to a predefined
// set of parameters (referred to by 'preset') and a given quality factor.
// This function can be called as a replacement to WebPConfigInit(). Will
// return 0 in case of error.
static WEBP_INLINE int WebPConfigPreset(WebPConfig* const config,
                                        WebPPreset preset, float quality) {
  return WebPConfigInitInternal(config, preset, quality,
                                WEBP_ENCODER_ABI_VERSION);
}

// Returns 1 if all parameters are in valid range and the configuration is OK.
WEBP_EXTERN(int) WebPValidateConfig(const WebPConfig* const config);

//------------------------------------------------------------------------------
// Input / Output

typedef struct WebPPicture WebPPicture;   // main structure for I/O

// non-essential structure for storing auxiliary statistics
typedef struct {
  float PSNR[4];          // peak-signal-to-noise ratio for Y/U/V/All
  int coded_size;         // final size
  int block_count[3];     // number of intra4/intra16/skipped macroblocks
  int header_bytes[2];    // approximate number of bytes spent for header
                          // and mode-partition #0
  int residual_bytes[3][4];  // approximate number of bytes spent for
                             // DC/AC/uv coefficients for each (0..3) segments.
  int segment_size[4];    // number of macroblocks in each segments
  int segment_quant[4];   // quantizer values for each segments
  int segment_level[4];   // filtering strength for each segments [0..63]

  int alpha_data_size;    // size of the transparency data
  int layer_data_size;    // size of the enhancement layer data

  void* user_data;        // this field is free to be set to any value and
                          // used during callbacks (like progress-report e.g.).
} WebPAuxStats;

// Signature for output function. Should return 1 if writing was successful.
// data/data_size is the segment of data to write, and 'picture' is for
// reference (and so one can make use of picture->custom_ptr).
typedef int (*WebPWriterFunction)(const uint8_t* data, size_t data_size,
                                  const WebPPicture* const picture);

// Progress hook, called from time to time to report progress. It can return 0
// to request an abort of the encoding process, or 1 otherwise if all is OK.
typedef int (*WebPProgressHook)(int percent, const WebPPicture* const picture);

typedef enum {
  // chroma sampling
  WEBP_YUV420 = 0,   // 4:2:0
  WEBP_YUV422 = 1,   // 4:2:2
  WEBP_YUV444 = 2,   // 4:4:4
  WEBP_YUV400 = 3,   // grayscale
  WEBP_CSP_UV_MASK = 3,   // bit-mask to get the UV sampling factors
  // alpha channel variants
  WEBP_YUV420A = 4,
  WEBP_YUV422A = 5,
  WEBP_YUV444A = 6,
  WEBP_YUV400A = 7,   // grayscale + alpha
  WEBP_CSP_ALPHA_BIT = 4   // bit that is set if alpha is present
} WebPEncCSP;

// Encoding error conditions.
typedef enum {
  VP8_ENC_OK = 0,
  VP8_ENC_ERROR_OUT_OF_MEMORY,            // memory error allocating objects
  VP8_ENC_ERROR_BITSTREAM_OUT_OF_MEMORY,  // memory error while flushing bits
  VP8_ENC_ERROR_NULL_PARAMETER,           // a pointer parameter is NULL
  VP8_ENC_ERROR_INVALID_CONFIGURATION,    // configuration is invalid
  VP8_ENC_ERROR_BAD_DIMENSION,            // picture has invalid width/height
  VP8_ENC_ERROR_PARTITION0_OVERFLOW,      // partition is bigger than 512k
  VP8_ENC_ERROR_PARTITION_OVERFLOW,       // partition is bigger than 16M
  VP8_ENC_ERROR_BAD_WRITE,                // error while flushing bytes
  VP8_ENC_ERROR_FILE_TOO_BIG,             // file is bigger than 4G
  VP8_ENC_ERROR_USER_ABORT,               // abort request by user
  VP8_ENC_ERROR_LAST                      // list terminator. always last.
} WebPEncodingError;

// maximum width/height allowed (inclusive), in pixels
#define WEBP_MAX_DIMENSION 16383

struct WebPPicture {
  // input
  WebPEncCSP colorspace;     // colorspace: should be YUV420 for now (=Y'CbCr).
  int width, height;         // dimensions (less or equal to WEBP_MAX_DIMENSION)
  uint8_t *y, *u, *v;        // pointers to luma/chroma planes.
  int y_stride, uv_stride;   // luma/chroma strides.
  uint8_t *a;                // pointer to the alpha plane
  int a_stride;              // stride of the alpha plane

  // output
  WebPWriterFunction writer;  // can be NULL
  void* custom_ptr;           // can be used by the writer.

  // map for extra information
  int extra_info_type;    // 1: intra type, 2: segment, 3: quant
                          // 4: intra-16 prediction mode,
                          // 5: chroma prediction mode,
                          // 6: bit cost, 7: distortion
  uint8_t* extra_info;    // if not NULL, points to an array of size
                          // ((width + 15) / 16) * ((height + 15) / 16) that
                          // will be filled with a macroblock map, depending
                          // on extra_info_type.

  // where to store statistics, if not NULL:
  WebPAuxStats* stats;

  // original samples (for non-YUV420 modes)
  uint8_t *u0, *v0;
  int uv0_stride;

  WebPEncodingError error_code;   // error code in case of problem.

  WebPProgressHook progress_hook;  // if not NULL, called while encoding.
};

// Internal, version-checked, entry point
WEBP_EXTERN(int) WebPPictureInitInternal(WebPPicture* const, int);

// Should always be called, to initialize the structure. Returns 0 in case of
// version mismatch. WebPPictureInit() must have succeeded before using the
// 'picture' object.
static WEBP_INLINE int WebPPictureInit(WebPPicture* const picture) {
  return WebPPictureInitInternal(picture, WEBP_ENCODER_ABI_VERSION);
}

//------------------------------------------------------------------------------
// WebPPicture utils

// Convenience allocation / deallocation based on picture->width/height:
// Allocate y/u/v buffers as per colorspace/width/height specification.
// Note! This function will free the previous buffer if needed.
// Returns 0 in case of memory error.
WEBP_EXTERN(int) WebPPictureAlloc(WebPPicture* const picture);

// Release memory allocated by WebPPictureAlloc() or WebPPictureImport*()
// Note that this function does _not_ free the memory pointed to by 'picture'.
WEBP_EXTERN(void) WebPPictureFree(WebPPicture* const picture);

// Copy the pixels of *src into *dst, using WebPPictureAlloc.
// Returns 0 in case of memory allocation error.
WEBP_EXTERN(int) WebPPictureCopy(const WebPPicture* const src,
                                 WebPPicture* const dst);

// self-crops a picture to the rectangle defined by top/left/width/height.
// Returns 0 in case of memory allocation error, or if the rectangle is
// outside of the source picture.
WEBP_EXTERN(int) WebPPictureCrop(WebPPicture* const picture,
                                 int left, int top, int width, int height);

// Rescale a picture to new dimension width x height.
// Now gamma correction is applied.
// Returns false in case of error (invalid parameter or insufficient memory).
WEBP_EXTERN(int) WebPPictureRescale(WebPPicture* const pic,
                                    int width, int height);

// Colorspace conversion function to import RGB samples.
// Previous buffer will be free'd, if any.
// *rgb buffer should have a size of at least height * rgb_stride.
// Returns 0 in case of memory error.
WEBP_EXTERN(int) WebPPictureImportRGB(
    WebPPicture* const picture, const uint8_t* const rgb, int rgb_stride);
// Same, but for RGBA buffer
WEBP_EXTERN(int) WebPPictureImportRGBA(
    WebPPicture* const picture, const uint8_t* const rgba, int rgba_stride);

// Variant of the above, but taking BGR(A) input:
WEBP_EXTERN(int) WebPPictureImportBGR(
    WebPPicture* const picture, const uint8_t* const bgr, int bgr_stride);
WEBP_EXTERN(int) WebPPictureImportBGRA(
    WebPPicture* const picture, const uint8_t* const bgra, int bgra_stride);

//------------------------------------------------------------------------------
// Main call

// Main encoding call, after config and picture have been initialized.
// 'picture' must be less than 16384x16384 in dimension (cf WEBP_MAX_DIMENSION),
// and the 'config' object must be a valid one.
// Returns false in case of error, true otherwise.
// In case of error, picture->error_code is updated accordingly.
WEBP_EXTERN(int) WebPEncode(
    const WebPConfig* const config, WebPPicture* const picture);

//------------------------------------------------------------------------------

#if defined(__cplusplus) || defined(c_plusplus)
}    // extern "C"
#endif

#endif  /* WEBP_WEBP_ENCODE_H_ */
