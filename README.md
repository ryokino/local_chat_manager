# Local Chat Messenger

A simple local chat application that performs Unix domain socket communication between a server and client, with fake data generation capabilities.

## Features

- Using Rust language
- Unix domain socket-based communication
- Support for both TCP and UDP protocols
- Fake data generation using the `fake` crate
- Support for different types of fake data:
  - User information (name and email)
  - Company names
  - Random quotes
  - Combined responses

## Prerequisites

- Rust (latest stable version)
- Cargo

## Usage

### TCP

#### Starting the TCP Server

```bash
cargo run --bin server
```

The server will start and listen for TCP connections on `/tmp/socket_file`.

#### Running the TCP Client

```bash
cargo run --bin client
```

### UDP

#### Starting the UDP Server

```bash
cargo run --bin udp_server
```

The server will start and listen for UDP connections on `/tmp/socket_file`.

#### Running the UDP Client

```bash
cargo run --bin udp_client
```

### Message Interaction (TCP only)

When prompted, enter a message. The server will respond with fake data based on your message content.

### Message Keywords

Try sending messages with these keywords to get different types of fake data:

- `name` or `user`: Generates a random name and email
- `company` or `business`: Generates a random company name
- `quote` or `text`: Generates a random sentence
- Any other message: Generates a combined response with name, company, and a random quote

## Example

```
Enter a message to send to the server:
> name

Server response: Generated user: John Smith <john.smith@example.com>

Enter a message to send to the server:
> company

Server response: Random company: Acme Corporation

Enter a message to send to the server:
> quote

Server response: Random quote: "The quick brown fox jumps over the lazy dog."
```

## Stopping the Server

Press `Ctrl+C` to stop the server. 