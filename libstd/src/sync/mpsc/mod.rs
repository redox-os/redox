mod mpsc_queue;
pub mod receiver;
pub mod sender;

pub use self::receiver::Receiver;
pub use self::sender::Sender;
use self::mpsc_queue::Queue;

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let sender = Sender {
        queue: box Queue::new(),
    };

    let receiver = Receiver {
        queue: &*sender.queue as *const _,
    };

    (sender, receiver)
}

// These tests are from the Rust repo. Their license applies:

/* Copyright (c) 2010-2011 Dmitry Vyukov. All rights reserved.
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 *
 *    1. Redistributions of source code must retain the above copyright notice,
 *       this list of conditions and the following disclaimer.
 *
 *    2. Redistributions in binary form must reproduce the above copyright
 *       notice, this list of conditions and the following disclaimer in the
 *       documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY DMITRY VYUKOV "AS IS" AND ANY EXPRESS OR IMPLIED
 * WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF
 * MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT
 * SHALL DMITRY VYUKOV OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT,
 * INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
 * LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
 * LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE
 * OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF
 * ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 *
 * The views and conclusions contained in the software and documentation are
 * those of the authors and should not be interpreted as representing official
 * policies, either expressed or implied, of Dmitry Vyukov.
 */
#[cfg(test)]
mod tests {
    use prelude::v1::*;

    use sync::mpsc::channel;
    use super::{Queue, Data, Empty, Invalid};
    use sync::Arc;
    use thread;

    #[test]
    fn test_full() {
        let q: Queue<Box<_>> = Queue::new();
        q.push(box 1);
        q.push(box 2);
    }

    #[test]
    fn test() {
        let nthreads = 8;
        let nmsgs = 1000;
        let q = Queue::new();
        match q.pop() {
            Empty => {}
            Invalid | Data(..) => panic!()
        }
        let (tx, rx) = channel();
        let q = Arc::new(q);

        for _ in 0..nthreads {
            let tx = tx.clone();
            let q = q.clone();
            thread::spawn(move|| {
                for i in 0..nmsgs {
                    q.push(i);
                }
                tx.send(()).unwrap();
            });
        }

        let mut i = 0;
        while i < nthreads * nmsgs {
            match q.pop() {
                Empty | Invalid => {},
                Data(_) => { i += 1 }
            }
        }
        drop(tx);
        for _ in 0..nthreads {
            rx.recv().unwrap();
        }
    }
}
