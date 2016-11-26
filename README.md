# rucrack
info security project 1

## Instructions

There are two arguments that you can pass to the Command, master and slave. Both versions require the user to pass in the hosts IP address.

The master needs to define:
- host `ip_address`
- `IV file` they want to use.
- optimal `thread_count`
- number of owned `slaves` passed by an array of ip addresses

The slave needs to define:
- host `ip_address`
- master `ip_address`
