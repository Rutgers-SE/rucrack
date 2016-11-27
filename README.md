# rucrack
info security project 1

## Instructions

CLONE & CD INTO REPO!!

```bash
$ git clone https://github.com/Rutgers-SE/rucrack && cd rucrack
```

There are two arguments that you can pass to the Command, master and slave. Both versions require the user to pass in the hosts IP address.

To run the master:

```bash
$ cargo run master 8080
master> iv-file <iv-filename>
master> cipher-file <cipher-filename>
master> slave http://x.x.x.x:xxx # This will change per slave
```

To run the slave:

```bash
$ cargo run slave http://x.x.x.x:xxxx
```

This will start a server and wait for jobs
