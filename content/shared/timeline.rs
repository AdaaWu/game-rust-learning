// ===== 時間軸小幫手：把事件印給右邊的時間軸看，不改變程式行為（不用改） =====
mod timeline {
    use std::cell::RefCell;
    use std::ops::{Deref, DerefMut};
    use std::panic::Location;
    use std::sync::mpsc::{Receiver, RecvError, SendError, Sender};
    use std::sync::{LockResult, Mutex, MutexGuard, PoisonError};

    thread_local! {
        static LANE: RefCell<Option<String>> = const { RefCell::new(None) };
    }
    // 印出 send 與真正送出包在一起，channel 的先後才對得上
    static SENDING: Mutex<()> = Mutex::new(());

    fn clean(s: &str) -> String {
        s.replace(['|', '\n', '\r'], " ")
    }

    fn emit(kind: &str, line: Option<u32>, target: &str) {
        if cfg!(test) {
            return;
        }
        let who = LANE.with(|l| l.borrow().clone()).unwrap_or_else(|| {
            let t = std::thread::current();
            t.name().map(clean).unwrap_or_else(|| format!("{:?}", t.id()))
        });
        let line = line.map(|n| n.to_string()).unwrap_or_default();
        println!("@tl|{}|{}|{}|{}", kind, line, who, clean(target));
    }

    // 替這條執行緒取名；回傳值被 drop 時記錄執行緒結束
    pub struct Lane;

    #[track_caller]
    pub fn lane(name: &str) -> Lane {
        LANE.with(|l| *l.borrow_mut() = Some(clean(name)));
        emit("start", Some(Location::caller().line()), "");
        Lane
    }

    impl Drop for Lane {
        fn drop(&mut self) {
            emit("end", None, "");
        }
    }

    // 與 mutex.lock() 用法相同，多一個鎖的名字；回傳值被 drop 時解鎖
    pub struct Held<'a, T> {
        guard: MutexGuard<'a, T>,
        name: String,
    }

    #[track_caller]
    pub fn lock<'a, T>(mutex: &'a Mutex<T>, name: &str) -> LockResult<Held<'a, T>> {
        let line = Some(Location::caller().line());
        emit("lock_wait", line, name);
        let result = match mutex.lock() {
            Ok(guard) => Ok(Held { guard, name: name.to_string() }),
            Err(p) => Err(PoisonError::new(Held { guard: p.into_inner(), name: name.to_string() })),
        };
        emit("lock_got", line, name);
        result
    }

    impl<T> Deref for Held<'_, T> {
        type Target = T;
        fn deref(&self) -> &T {
            &self.guard
        }
    }

    impl<T> DerefMut for Held<'_, T> {
        fn deref_mut(&mut self) -> &mut T {
            &mut self.guard
        }
    }

    impl<T> Drop for Held<'_, T> {
        fn drop(&mut self) {
            emit("lock_free", None, &self.name);
        }
    }

    // 與 tx.send(value) 用法相同，多一個 channel 的名字
    #[track_caller]
    pub fn send<T>(tx: &Sender<T>, name: &str, value: T) -> Result<(), SendError<T>> {
        if cfg!(test) {
            return tx.send(value);
        }
        let _order = SENDING.lock().unwrap_or_else(PoisonError::into_inner);
        emit("send", Some(Location::caller().line()), name);
        tx.send(value)
    }

    // 與 rx.recv() 用法相同，多一個 channel 的名字
    #[track_caller]
    pub fn recv<T>(rx: &Receiver<T>, name: &str) -> Result<T, RecvError> {
        let line = Some(Location::caller().line());
        emit("recv_wait", line, name);
        let result = rx.recv();
        emit(if result.is_ok() { "recv" } else { "recv_closed" }, line, name);
        result
    }

    // 在時間軸上插一面小旗子
    #[track_caller]
    pub fn note(text: &str) {
        emit("note", Some(Location::caller().line()), text);
    }
}
