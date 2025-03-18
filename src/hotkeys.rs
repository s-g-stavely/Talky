use anyhow::Result;
use global_hotkey::{
    hotkey::{HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager,
};
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use winit::event_loop::{ControlFlow, EventLoop};

pub struct HotkeyListener {
    hotkey_manager: GlobalHotKeyManager,
    recording_state: Arc<AtomicBool>,
    hotkey_id: u32,
}

impl HotkeyListener {
    pub fn new() -> Result<Self> {
        let hotkey_manager = GlobalHotKeyManager::new()?;
        let recording_state = Arc::new(AtomicBool::new(false));
        
        Ok(Self {
            hotkey_manager,
            recording_state,
            hotkey_id: 1, // Default ID
        })
    }
    
    pub fn setup_hotkey(&mut self) -> Result<()> {
        // Define Ctrl+Shift+Space hotkey
        let hotkey = HotKey::new(
            Some(Modifiers::CONTROL | Modifiers::SHIFT),
            global_hotkey::hotkey::Code::Space,
        );
        
        self.hotkey_id = hotkey.id();
        self.hotkey_manager.register(hotkey)?;
        
        Ok(())
    }
    
    pub fn get_recording_state(&self) -> Arc<AtomicBool> {
        self.recording_state.clone()
    }
        
    // Runs the hotkey listener event loop. This call will block.
    pub fn run(self) -> Result<()> {
        let recording_state = self.recording_state.clone();
        let event_loop = EventLoop::new()?;
        let hotkey_id = self.hotkey_id;
        
        // Important: Create the receiver after registering hotkeys
        let hotkey_channel = GlobalHotKeyEvent::receiver();

        event_loop.set_control_flow(ControlFlow::Wait);
        
        event_loop.run(move |_, _| {
            
            if let Ok(event) = hotkey_channel.try_recv() {
                if event.id == hotkey_id && event.state() == global_hotkey::HotKeyState::Pressed {
                    // Toggle recording state when hotkey is pressed
                    let current_state = recording_state.load(Ordering::SeqCst);
                    let new_state = !current_state;
                    recording_state.store(new_state, Ordering::SeqCst);
                    
                    println!("Hotkey pressed! Recording state changed to: {}", new_state);
                }
            }
        })?;
        
        Ok(())
    }
}
