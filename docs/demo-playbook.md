terminal: 77x24
white background

## Preparation
```sh
title fpick
cd
tput reset
```

screenkey
```sh
screenkey --bg-color="#333" --opacity=0.5 --timeout=0.5
```
gnome screen recorder

## Action
```sh
cd "$(fpick)"
/
op [Enter]
g,am [Space] li [Enter]
s,t [Enter]
[Down] (steamapps) [Enter]
[Down][Down][Up] (common) [Enter]
fa [Enter]
b [Enter]
x [Enter]
[Enter]
pwd
```

Record GIF

## Post prod
Convert MP4 to GIF
```sh
ffmpeg -i demo.mp4 -vf "fps=10,scale=640:-1:flags=lanczos,palettegen" palette.png
ffmpeg -i demo.mp4 -i palette.png -filter_complex "fps=10,scale=640:-1:flags=lanczos[x];[x][1:v]paletteuse" demo.gif
```
