use syncer_core::ClipboardPayload;

pub trait ClipboardAdapter {
    fn read_current(&self) -> ClipboardPayload;
    fn write_remote_content(&mut self, payload: ClipboardPayload);
    fn set_local_content(&mut self, content: impl Into<String>);
}

#[derive(Default)]
pub struct MemoryClipboard {
    current: String,
}

impl ClipboardAdapter for MemoryClipboard {
    fn read_current(&self) -> ClipboardPayload {
        ClipboardPayload {
            content: self.current.clone(),
        }
    }

    fn write_remote_content(&mut self, payload: ClipboardPayload) {
        log::info!("应用远端剪切板内容");
        self.current = payload.content;
    }

    fn set_local_content(&mut self, content: impl Into<String>) {
        self.current = content.into();
        log::debug!("本地剪切板已更新");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_and_reads_content() {
        let mut clipboard = MemoryClipboard::default();
        clipboard.set_local_content("hi");
        assert_eq!(clipboard.read_current().content, "hi");
    }
}
