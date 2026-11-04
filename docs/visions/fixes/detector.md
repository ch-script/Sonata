# Detector module on Core-Abs must be updated

Mainly because we dont fully support BSD systems yet. 
The following need updating:

- GPU Detection: Because its linux exclusive, it should use commands like "pciconf -lv | grep -i vga" on BSD
