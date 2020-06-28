import subprocess
for i in range(0, 314):
    pic_ordering = "0000%d" % (i)
    subprocess.run("./target/release/rust-tracer %d > frame_%s.ppm" % (i, pic_ordering[-4:]), shell=True, check=True)

