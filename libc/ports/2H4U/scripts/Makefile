#! /bin/sh

OBJECTS	= ../objs/Engine.o ../objs/bonus.o ../objs/highscore.o ../objs/launcher.o ../objs/son.o ../objs/Param.o ../objs/brick.o ../objs/intro.o ../objs/main.o ../objs/terrain.o ../objs/balle.o ../objs/figure.o ../objs/jeu.o ../objs/menu.o
TARGET	=	../2H4U
LIBS	=	-lSDL_image -lSDL_mixer -lSDL_ttf -lSDL -lfreetype -lpng -lz

all: dirs $(OBJECTS)
	$(CXX) -o $(TARGET) $(OBJECTS) $(LIBS)

dirs:
	mkdir -p ../objs

../objs/Engine.o: ../sources/Engine.cpp
	$(CXX) -o $@ -c $<

../objs/bonus.o: ../sources/bonus.cpp
	$(CXX) -o $@ -c $<

../objs/highscore.o: ../sources/highscore.cpp
	$(CXX) -o $@ -c $<

../objs/launcher.o: ../sources/launcher.cpp
	$(CXX) -o $@ -c $<

../objs/son.o: ../sources/son.cpp
	$(CXX) -o $@ -c $<

../objs/Param.o: ../sources/Param.cpp
	$(CXX) -o $@ -c $<

../objs/brick.o: ../sources/brick.cpp
	$(CXX) -o $@ -c $<

../objs/intro.o: ../sources/intro.cpp
	$(CXX) -o $@ -c $<

../objs/main.o: ../sources/main.cpp
	$(CXX) -o $@ -c $<

../objs/terrain.o: ../sources/terrain.cpp
	$(CXX) -o $@ -c $<

../objs/balle.o: ../sources/balle.cpp
	$(CXX) -o $@ -c $<

../objs/figure.o: ../sources/figure.cpp
	$(CXX) -o $@ -c $<

../objs/jeu.o: ../sources/jeu.cpp
	$(CXX) -o $@ -c $<

../objs/menu.o: ../sources/menu.cpp
	$(CXX) -o $@ -c $<

rebuild:
	distclean
	all

clean:
	rm -rf $(OBJECTS)

distclean : clean
	rm -rf $(TARGET)
