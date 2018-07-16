# Hijack this file to set this PS1, visually indicating to the user that we are running the docker container

PS1="\[\e]0;\u@\h: \w\a\]${debian_chroot:+($debian_chroot)}\[\033[01;32m\]\u@\h\[\033[1;35m\]<$IMAGE_NAME>\[\033[00m\]:\[\033[01;34m\]\w\[\033[00m\]\$ "
