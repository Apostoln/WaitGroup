# WaitGroup

Crate for synchronization primitive "Wait Group" in Rust.
See [go/sync/WaitGroup](https://golang.org/pkg/sync/#WaitGroup).

Rust-docs and manual in English will be soon.

## Wait groups
- ManualWaitGroup -- go-like Wait Group with manual processing of internal counter.
- GuardWaitGroup -- RAII-wrapper for ManualWaitGroup
- SmartWaitGroup -- Wait group with separate Waiter and Doer.

## Manual
[Manual [ru]](https://github.com/Apostoln/WaitGroup/blob/master/manual_ru.md)
