use anyhow::Result;
use niri_ipc::{Action, ColumnDisplay, Request, Response, SizeChange};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

struct NiriClient {
    reader: BufReader<UnixStream>,
    writer: UnixStream,
}

impl NiriClient {
    fn new(socket_path: &str) -> Result<Self> {
        let stream = UnixStream::connect(socket_path)?;
        let reader = BufReader::new(stream.try_clone()?);
        let writer = stream;

        Ok(NiriClient { reader, writer })
    }

    fn action(&mut self, action: Action) -> Result<niri_ipc::Reply> {
        self.execute(Request::Action(action))
    }
    fn execute(&mut self, request: Request) -> Result<niri_ipc::Reply> {
        writeln!(self.writer, "{}", serde_json::to_string(&request)?)?;

        let mut response_line = String::new();
        self.reader.read_line(&mut response_line)?;

        Ok(serde_json::from_str(&response_line)?)
    }

    fn get_windows(&mut self) -> Result<Vec<niri_ipc::Window>> {
        match self.execute(Request::Windows)? {
            Ok(Response::Windows(windows)) => Ok(windows),
            _ => Err(anyhow::anyhow!("Failed to get windows")),
        }
    }

    fn get_workspaces(&mut self) -> Result<Vec<niri_ipc::Workspace>> {
        match self.execute(Request::Workspaces)? {
            Ok(Response::Workspaces(workspaces)) => Ok(workspaces),
            _ => Err(anyhow::anyhow!("Failed to get workspaces")),
        }
    }
}

fn main() -> Result<()> {
    let socket_path = std::env::var("NIRI_SOCKET")
        .map_err(|_| anyhow::anyhow!("NIRI_SOCKET environment variable not set"))?;

    let mut client = NiriClient::new(&socket_path)?;

    let windows = client.get_windows()?;
    let workspaces = client.get_workspaces()?;

    let current_workspace = workspaces
        .iter()
        .find(|ws| ws.is_focused)
        .ok_or_else(|| anyhow::anyhow!("No focused workspace found"))?;

    // Get current workspace windows
    let current_workspace_windows: Vec<_> = windows
        .iter()
        .filter(|w| w.workspace_id == Some(current_workspace.id))
        .collect();

    if current_workspace_windows.is_empty() {
        println!("No windows found on current workspace");
        return Ok(());
    }

    let window_count = current_workspace_windows.len();
    println!("✅ Found {} windows on current workspace", window_count);

    if window_count == 0 {
        println!("❌ No windows to arrange");
        return Ok(());
    }

    let num_columns = num_columns(window_count);
    let windows_per_column = (window_count + num_columns - 1) / num_columns; // Ceiling division
    let column_width = 100.0 / num_columns as f64;

    println!(
        "📐 Creating {} columns with up to {} windows per column",
        num_columns, windows_per_column
    );

    for (_i, window) in current_workspace_windows.iter().enumerate() {
        let _ = client.action(Action::FocusWindow { id: window.id })?;
        let _ = client.action(Action::ExpelWindowFromColumn {})?;
    }

    // Process each column from left to right
    for column_idx in 0..num_columns {
        let start_window = column_idx * windows_per_column;
        let end_window = ((column_idx + 1) * windows_per_column).min(window_count);

        if start_window >= window_count {
            println!("   ✅ Column {}: No more windows to process", column_idx);
            break; // No more windows to process
        }

        println!(
            "🏛️  Building column {} with windows {}-{}",
            column_idx,
            start_window,
            end_window - 1
        );

        let _ = client.action(Action::FocusColumn {
            index: column_idx + 1,
        })?;
        let _ = client.action(Action::SetColumnDisplay {
            display: ColumnDisplay::Normal,
        })?;
        let _ = client.action(Action::SetWindowWidth {
            id: None,
            change: SizeChange::SetProportion(column_width),
        })?;

        // Consume additional windows into this column
        let windows_to_consume = end_window - start_window - 1;
        for _i in 0..windows_to_consume {
            let _ = client.action(Action::ConsumeWindowIntoColumn {})?;
        }
    }

    let _ = client.action(Action::FocusColumnFirst {})?;

    println!(
        "✅ Successfully arranged {} windows into {} columns!",
        window_count, num_columns
    );

    Ok(())
}

fn num_columns(window_count: usize) -> usize {
    if window_count == 0 {
        return 1;
    }

    let columns = (window_count as f64).sqrt().ceil() as usize;

    columns.min(window_count)
}
