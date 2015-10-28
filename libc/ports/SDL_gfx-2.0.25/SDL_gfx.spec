%define prefix  %{_prefix}
%define version 2.0.25
%define release 1
%define _unpackaged_files_terminate_build 0

Summary: SDL graphics drawing primitives and other support functions
Name: SDL_gfx
Version: %{version}
Release: %{release}
License: ZLIB
Group: System Environment/Libraries
Prefix: %{prefix}
Source: http://www.ferzkopp.net/Software/SDL_gfx-2.0/SDL_gfx-2.0.25.tar.gz
Packager: Danny Sung <dannys at mail.com>
Vendor: Andreas Schiffler <aschiffler at ferzkopp.net>
BuildRoot: /tmp/%{name}-root-%{version}

%description
The SDL_gfx library evolved out of the SDL_gfxPrimitives code which
provided basic drawing routines such as lines, circles or polygons and
SDL_rotozoom which implemented a interpolating rotozoomer for SDL
surfaces.

The current components of the SDL_gfx library are:

   * Graphic Primitives (SDL_gfxPrimitves.h)
   * Rotozoomer (SDL_rotozoom.h)
   * Framerate control (SDL_framerate.h)
   * MMX image filters (SDL_imageFilter.h)
   * Custom blit functions (SDL_gfxBlitFunc.h)

The library is is written in plain C and can be used in C++ code.

%package devel
Summary: Libraries and includes to develop SDL_gfx programs
Group: Development/Libraries
Requires: %{name} = %{version}

%description devel
The SDL_gfx library evolved out of the SDL_gfxPrimitives code which
provided basic drawing routines such as lines, circles or polygons and
SDL_rotozoom which implemented a interpolating rotozoomer for SDL
surfaces.

The current components of the SDL_gfx library are:

   * Graphic Primitives (SDL_gfxPrimitves.h)
   * Rotozoomer (SDL_rotozoom.h)
   * Framerate control (SDL_framerate.h)
   * MMX image filters (SDL_imageFilter.h)
   * Custom blit functions (SDL_gfxBlitFunc.h)

The library is is written in plain C and can be used in C++ code.

%package demos
Summary: SDL_gfx demo and test programs
Group: Applications/Multimedia
Requires: %{name} = %{version}

%description demos
SDL_gfx demo applications and source code.

%prep
%setup -q

%build
./autogen.sh
# aclocal
%define _includedir /usr/include
%configure
CFLAGS=$RPM_OPT_FLAGS make

cd Test 
CFLAGS="-I../" LDFLAGS="-L../.libs/" ./configure
make
cd ..

%install
%makeinstall

install -m755 -d $RPM_BUILD_ROOT%{_datadir}/SDL_gfx-demos
cp Test/* $RPM_BUILD_ROOT%{_datadir}/SDL_gfx-demos

%clean
rm -rf $RPM_BUILD_ROOT

%files
%defattr(-,root,root)
%{_libdir}/libSDL_gfx.so*

%files devel
%defattr(-,root,root)
%doc AUTHORS COPYING ChangeLog INSTALL LICENSE NEWS README
%doc Docs/
%{_libdir}/libSDL_gfx.a
%{_libdir}/libSDL_gfx.la
%{_includedir}/SDL/SDL_framerate.h
%{_includedir}/SDL/SDL_gfxPrimitives_font.h
%{_includedir}/SDL/SDL_gfxPrimitives.h
%{_includedir}/SDL/SDL_imageFilter.h
%{_includedir}/SDL/SDL_rotozoom.h
%{_includedir}/SDL/SDL_gfxBlitFunc.h

%files demos
%defattr(-,root,root)
%{_datadir}/SDL_gfx-demos

%changelog
