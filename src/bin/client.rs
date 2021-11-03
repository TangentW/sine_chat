use guard::guard;
use sine_chat::{
    frame,
    message::{ClientMessage, Content, Handshake, HandshakeReply, MessageReply, ServerMessage},
};
use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader},
    net::TcpStream,
};
use tokio_util::either::Either;

const ADDR: &str = "127.0.0.1:8888";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Please input your name:");
    let user_name = input().await;
    let (reader, writer) = connect_then_handshake(user_name).await?;
    let send_task = tokio::spawn(send_message(writer));
    receive_message(reader).await;
    println!("Disconnected");
    send_task.abort();
    Ok(())
}

type Reader = frame::Reader<tokio::net::tcp::OwnedReadHalf>;
type Writer = frame::Writer<tokio::net::tcp::OwnedWriteHalf>;

async fn connect_then_handshake(user_name: String) -> anyhow::Result<(Reader, Writer)> {
    let stream = TcpStream::connect(&ADDR).await?;

    let (reader, writer) = stream.into_split();
    let mut reader = Reader::new(reader);
    let mut writer = Writer::new(writer);

    println!("Handshake ...");
    let handshake = Handshake::new(user_name);
    writer.write(handshake).await?;

    let reply = reader
        .read::<HandshakeReply>()
        .await
        .map(|x| x.map_err(|e| e.into()))
        .unwrap_or(Err(anyhow::Error::msg("Expect handshake reply")))?;

    if reply.success {
        println!("Handshake completed");
        Ok((reader, writer))
    } else {
        Err(anyhow::Error::msg(
            reply.message.unwrap_or("Unknown".into()),
        ))
    }
}

async fn input() -> String {
    let mut input = String::new();
    let mut reader = BufReader::new(stdin());
    reader.read_line(&mut input).await.unwrap();
    input.trim().into()
}

async fn send_message(mut writer: Writer) {
    loop {
        let input = input().await;
        guard!(let Some((receiver, text)) = input.split_once("<")
        else {
            println!("Invalid input!");
            continue;
        });
        // Send message.
        let message = ClientMessage::new(Content::Text(text.trim().into()), receiver.trim().into());
        if let Err(err) = writer.write(message).await {
            eprintln!("Sending error: {}", err);
        }
    }
}

async fn receive_message(mut reader: Reader) {
    while let Some(msg) = reader.read_either::<ServerMessage, MessageReply>().await {
        match msg {
            Ok(msg) => match msg {
                Either::Left(msg) => {
                    println!("[{} > {}] {}", msg.sender, msg.receiver, msg.content);
                }
                Either::Right(reply) if !reply.success => {
                    eprintln!(
                        "Sending error: {}",
                        reply.message.unwrap_or("Unknown".into())
                    );
                }
                _ => (),
            },
            Err(err) => eprintln!("Receiving error: {}", err),
        }
    }
}
