use std::{
    env,
    io::{stdout, Write},
    time::Duration,
};

use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, DisableMouseCapture, Event as CEvent, KeyCode, KeyEvent},
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    ExecutableCommand,
};
use futures::{pin_mut, StreamExt};
use ndgr_client::{
    comment_buffer::CommentBuffer, fetch_program_info, stream_chunked_message,
    websocket::WebSocketClient,
};
use tokio::{select, sync::mpsc};

#[tokio::main]
async fn main() -> Result<()> {
    let url = env::args().nth(1).expect("URL is required");

    let info = fetch_program_info(&url).await.unwrap();

    let web_socket_client = WebSocketClient::new(&info.site.relive.web_socket_url)
        .await
        .unwrap();
    let view_uri = web_socket_client.view_uri();

    println!("view_uri: {}", view_uri);

    let stream = stream_chunked_message(view_uri).await;
    pin_mut!(stream);

    enable_raw_mode()?;
    let mut stdout = stdout();

    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Clear(ClearType::All))?;
    stdout.flush()?;

    let (width, height) = crossterm::terminal::size()?;

    let mut comment_buffer = CommentBuffer::new(width as usize, height as usize - 2);

    let (tx, mut rx) = mpsc::unbounded_channel();

    tokio::spawn(async move {
        loop {
            if event::poll(Duration::from_millis(100)).unwrap() {
                if let CEvent::Key(KeyEvent { code, .. }) = event::read().unwrap() {
                    if tx.send(code).is_err() {
                        break;
                    }
                }
            }
        }
    });

    let mut input = String::new();

    loop {
        stdout.execute(cursor::Hide)?;
        stdout.execute(cursor::MoveTo(0, 0))?;

        for (i, comment) in comment_buffer.comments().iter().enumerate() {
            stdout.execute(cursor::MoveTo(0, i as u16)).unwrap();
            write!(stdout, "{}", comment).unwrap();
            stdout.execute(Clear(ClearType::UntilNewLine))?;
        }

        stdout.execute(cursor::MoveTo(0, height - 1))?;
        stdout.execute(Clear(ClearType::CurrentLine))?;
        print!("コメント入力: {}", input);
        stdout.flush()?;

        select! {
            message = stream.next() => {
                if let Some(message) = message {
                    let msg_str = format!("{:?}", message);
                    comment_buffer.push(msg_str);
                } else {
                    break;
                }
            },
            Some(key) = rx.recv() => {
                match key {
                    KeyCode::Char(c) => {
                        input.push(c);
                    },
                    KeyCode::Backspace => {
                        input.pop();
                    },
                    KeyCode::Enter => {
                        if !input.is_empty() {
                            // TODO
                            comment_buffer.push(format!("あなた: {}", input));
                            input.clear();
                        }
                    },
                    KeyCode::Esc => {
                        break;
                    },
                    _ => {}
                }
            },
        }

        stdout.execute(cursor::Show)?;
    }

    disable_raw_mode()?;
    stdout.execute(LeaveAlternateScreen)?;
    stdout.execute(DisableMouseCapture)?;
    stdout.execute(cursor::Show)?;
    Ok(())
}
