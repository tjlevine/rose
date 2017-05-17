# Rose: The Rust Operating System for (my own) Education

## What is Rose?

Rose is a hobbyist operating system which I'm developing for a couple reasons:
1) I want to learn Rust better, especially starting from a `#![no_std]` context
2) I want to learn more about low level OS internals, especially on OS's targeting x86_64

Fortunately, there are some excellent resources available to get started, with the most important being Philipp 
Oppermann's fantastic blog series [Writing an OS in Rust](http://os.phil-op.com), which I'm following quite closely at
this early stage. Also, I would be remiss if I didn't also mention the great 
[OSDev Wiki](http://wiki.osdev.org/Main_Page), which is full of priceless OS-related wisdom.

## Building and Running Rose

Currently, the easiest way to build and run Rose is to use the provided Dockerfile to create a containerized
environment for Rose to run in.

First, clone the repository and make sure your docker install is working.

Then, you can build the docker container:

```
docker build -t rose:latest .
```

and then run it:
```
docker run -it rose:latest
```

Currently, Rose just prints some debug information then halts the CPU. If you see "OS returned!" in the top left, you
know it's working correctly.

To exit from the QEMU process which is running Rose, press ESC and then 2. This should bring you to the QEMU monitor,
where you can type `q` then press Enter to exit QEMU. To return to Rose from the QEMU monitor, press Esc and then 1.

## Future Plans

Currently, I'd like to finish implementing the features detailed in Philipp's blog series, then work on a basic
terminal CLI so that a user can actually interact with Rose via a keyboard. Then I'll explore adding a
basic TCP/IP network stack, which will require writing a NIC driver of some kind. After that, I would like to start
on a basic filesystem. I'll do my best to document my work in case someone has any interest in following it later.

## License

[MIT](License.txt)
