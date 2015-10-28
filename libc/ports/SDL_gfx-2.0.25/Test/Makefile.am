AUTOMAKE_OPTIONS = no-dependencies
ACLOCAL_AMFLAGS = -I m4

bin_PROGRAMS = \
	TestGfxPrimitives \
	TestRotozoom \
	TestFramerate \
	TestImageFilter \
	TestFonts \
	TestABGR \
	TestShrink \
	TestGfxTexture \
	TestGfxBlit

TestGfxPrimitives_SOURCES = TestGfxPrimitives.c
TestRotozoom_SOURCES = TestRotozoom.c
TestFramerate_SOURCES = TestFramerate.c
TestImageFilter_SOURCES = TestImageFilter.c
TestFonts_SOURCES = TestFonts.c
TestABGR_SOURCES = TestABGR.c
TestShrink_SOURCES = TestShrink.c
TestGfxTexture_SOURCES = TestGfxTexture.c
TestGfxBlit_SOURCES = TestGfxBlit.c

DISTCLEANFILES = *~ *~c *~h *.cross.cache inc

distclean-local:
	-rm -rf autom4te.cache
	-rm -f *.user
	
