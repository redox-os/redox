GIT=https://github.com/redox-os/pastel.git
BINDIR=/ui/bin

function recipe_stage {
	mkdir "$1/ui"
	cp -rv res "$1/ui/pastel"
	mkdir "$1/ui/apps"
	cat > "$1/ui/apps/pastel" <<-EOF
	name=Pastel
	binary=/ui/bin/pastel
	icon=/ui/pastel/accessories-bitmap-editor.png
	accept=*.bmp
	accept=*.jpg
	accept=*.jpeg
	accept=*.png
	author=Robby Cerantola
	description=Bitmap Editor
	EOF
}
