The English version of this manual will be later.

# WaitGroup в языке Go
В языке Go есть такой примитив синхронизации, как [WaitGroup](https://golang.org/pkg/sync/#WaitGroup). Он позволяет описывать, каким образом одна из горутин (легковесный поток) должна ожидать выполнение других, чтобы продолжить свою работу. 

Рассмотрим пример:
```go
package main

import (
	"fmt"
	"sync"
	"sync/atomic"
)

func processCounter(counter *uint64, wg *sync.WaitGroup) {
	atomic.AddUint64(counter, 1)
	wg.Done() // Заканчиваем одну задачу
}

func main() {
	var counter uint64
	var wg sync.WaitGroup
	for i := 0; i < 100; i++ {
		wg.Add(1) // Добавляем еще одну задачу на ожидание
		go processCounter(&counter, &wg)
	}
	wg.Wait() // Ожидаем завершения всех задач
	fmt.Printf("%d", atomic.LoadUint64(&counter)) //100
}
```

Здесь мы создаем 100 горутин, в которых делаем некую полезную работу (инкрементируем какой-то счетчик). При этом, главная горутина продолжит свое выполнение, только когда все остальные закончат свое выполнение. 
WaitGroup хранит в себе счетчик текущих задач и имеет три метода:
```go
func (wg *WaitGroup) Add(delta int)
func (wg *WaitGroup) Done()
func (wg *WaitGroup) Wait()
```
`Add` изменяет внутренний счетчик на `delta`, `Done` его уменьшает на 1, а `Wait` останавливает выполнение, пока счетчик не станет равен 0. Все просто.
Обратите внимание, что `Add` принимает знаковое целое, и может уменьшать счетчик. Если счетчик окажется меньше нуля, `Add` запаникует, так же как и `Done` .

К сожалению, такого удобного примитива синхронизации нет в Rust. С этой целью я написал крейт [WaitGroup](https://github.com/Apostoln/WaitGroup).

# ManualWaitGroup
## Использование
Рассмотрим тот же пример, но со структурой [ManualWaitGroup](https://github.com/Apostoln/WaitGroup/blob/master/src/manual_wait_group.rs), которая просто копирует интерфейс гошной WaitGroup.

```Rust
use wait_group::ManualWaitGroup;  
  
fn process_counter(counter: Arc<AtomicIsize>, wg: ManualWaitGroup) {  
	counter.fetch_add(1, Ordering::SeqCst);  
	wg.done(); // Заканчиваем одну задачу
}  
  
fn main() {  
	let counter = Arc::new(AtomicIsize::new(0));  
	let wg = ManualWaitGroup::new();  
	for _ in 0..100 {  
	        let wg = wg.clone();
	        wg.add(1); // Добавляем еще одну задачу на ожидание
		let counter = Arc::clone(&counter);  
		thread::spawn(move || process_counter(counter, wg));  
	}  
	wg.wait(); // Ожидаем завершения всех задач
	println!("{}", counter.load(Ordering::SeqCst)); //100  
}
```

## Реализация ManualWaitGroup
ManualWaitGroup - это обвертка, которая хранит в себе умный указатель [Arc](https://doc.rust-lang.org/std/sync/struct.Arc.html) на имплементацию и предоставляет нужный нам go-образный интерфейс.
```Rust
pub struct ManualWaitGroup {  
	inner: Arc<WaitGroupImpl>,  
}
```

Его методы просто делегируют вызовы к имплементации ([Эта фича](https://hackmd.io/ZUEHoEgwRF29hbcIyUXIiw?view) хорошенько бы сократила код!):
```Rust
pub fn add(&self, delta: isize) {  
    self.inner.add(delta);  
}

pub fn done(&self) {  
    self.inner.done();  
}

pub fn wait(&self) {  
    self.inner.wait();  
}
```


## Реализация WaitGroupImpl

А вот сам WaitGroupImpl:
```Rust
pub struct WaitGroupImpl {  
	counter: Mutex<usize>,
	condition: Condvar,  
}
```
Как мы видим, у нас есть внутренний счетчик активных задач под [мутексом](https://doc.rust-lang.org/std/sync/struct.Mutex.html) и [условная переменная](https://doc.rust-lang.org/std/sync/struct.Condvar.html).

Упрощенно, имплементация выглядит так:
В момент вызова add() мы захватываем мутекс и пытаемся изменить счетчик. Если он оказывается меньше нуля, то кидаем панику. Если нет, то изменяем его. Если счетчик оказался равен нулю (Т.е. если все задачи завершились), то вызываем функцию notify_all(), которая пробудит все ожидающие потоки.
```Rust
pub fn add(&self, delta: isize) {  
	let mut count = self.counter.lock().unwrap();  
	let res = *count as isize + delta;  
	if res < 0 {  
		panic!("Negative counter");
	}
	*count = res as usize;  
	if *count == 0 {  
		self.condition.notify_all();  
	}  
}
```
<br />

Метод `done()` просто уменьшает счетчик на 1:
```Rust
pub fn done(&self) {  
	self.add(-1);
}
```
<br />

Наконец, `wait()`  останавливает выполнение текущего потока до тех пор, пока счетчик не станет равен 0 и об этом не прийдет уведомление через `.notify_all()`.  Этот поток будет усыплен, т.е. не будет занимать процессорное время.
```Rust
pub fn wait(&self) {  
	let mut count = self.counter.lock().unwrap();  
	while *count > 0 {  
	        count = self.condition.wait(count).unwrap();  
	}  
}
```
<br />

Однако, кидать панику не всегда приемлимо, поэтому и у `WaitGroupImpl` и у `ManualWaitGroup` есть  `try_*` варианты методов, которые вернут `Result<(), WaitGroupError>`:
```Rust
#[must_use]
pub fn try_add(&self, delta: isize) -> Result<()>;
#[must_use]
pub fn try_done(&self) -> Result<();
```
Они помечены с помощью `#[must_use]`, чтобы конечный пользователь не забыл обработать возвращаемую ошибку. 

## Преимущества ManualWaitGroup
Go-подобный интерфейс `ManualWaitGroup` позволяет вручную изменять внутренний счетчик любым необходимым образом. Возвращаясь к начальному примеру, мы можем, например, зараннее добавить 100 задач. 
```Rust
//...
let wg = ManualWaitGroup::new();  
wg.add(100);  
for _ in 0..100 {  
	let wg = wg.clone();  
	let counter = Arc::clone(&counter);  
	thread::spawn(move || process_counter(counter, wg));  
}
//...
```
Или же, мы можем написать произвольную более сложную логику, вызывая `add()` с некоторым отрицательным значением вместо вызова `done()`.

## Недостатки ManualWaitGroup
Однако, этот же интерфейс позволяет делать ошибки:
### Bug 1.1: `add()` больше чем `done()`
Если мы увеличим счетчик на значение, которое больше чем количество вызовов `done()`, получим дедлок при вызове `wait()`
```Rust
use wait_group::ManualWaitGroup;  
  
fn process_counter(counter: Arc<AtomicIsize>, wg: ManualWaitGroup) {  
	counter.fetch_add(1, Ordering::SeqCst);  
	wg.done(); 
}  
  
fn main() {  
	let counter = Arc::new(AtomicIsize::new(0));  
	let wg = ManualWaitGroup::new();  
	wg.add(101); // Создаем больше задач, чем спавним потоков
	for _ in 0..100 {  
		let wg = wg.clone();  
		let counter = Arc::clone(&counter);  
		thread::spawn(move || process_counter(counter, wg));  
	}
	wg.wait(); // Получаем дедлок
	println!("{}", counter.load(Ordering::SeqCst));  
}
```

### Bug #1.2: `done()` больше чем `add()`
Аналогично, мы можем случайно получить больше вызовов `done()` чем изначально добавили в `add()`. В таком случае мы получим панику в момент вызова "лишнего" `done()`
```Rust
use wait_group::ManualWaitGroup;  
  
fn process_counter(counter: Arc<AtomicIsize>, wg: ManualWaitGroup) {  
	counter.fetch_add(1, Ordering::SeqCst);  
	wg.done(); // Паника
}  
  
fn main() {  
	let counter = Arc::new(AtomicIsize::new(0));  
	let wg = ManualWaitGroup::new();  
	wg.add(99);  
	for _ in 0..100 {  
		let wg = wg.clone();  
		let counter = Arc::clone(&counter);  
		thread::spawn(move || process_counter(counter, wg));  
	}
	wg.wait(); 
	println!("{}", counter.load(Ordering::SeqCst));
}
```

### Bug #1.3: Забытый вызов `add()` или `done()`
Даже используя "явный" вызов `add(1)` вместо "неявного" `add(100)`, мы можем легко забыть вызвать `done()` или этот самый `add(1)`, получив один из двух вышеописанных багов


## Ну и что?
Можно, конечно, сказать, что это слишком очевидные ошибки, которые хороший программист совершать не должен. Однако, во-первых, реальный код может быть существенно сложнее и неочевиднее, а во-вторых, даже хороший программист может быть уставшим и невнимательным.
К тому же,  подход "сам виноват" противоречит философии языка Rust, который параноидально стремится всеми силами уберечь нас от ошибок.

# GuardWaitGroup
## Использование
В расте у нас есть замечательная идиома [RAII](https://en.wikipedia.org/wiki/Resource_acquisition_is_initialization). Воспользуемся ею и будем увеличивать внутренний счетчик в конструкторе и уменьшать в деструкторе. Получится структура [GuardWaitGroup](https://github.com/Apostoln/WaitGroup/blob/master/src/guard_wait_group.rs), с помощью которой можно переписать предыдущий пример следующим образом:

```Rust
use wait_group::GuardWaitGroup;  
  
fn process_counter(counter: Arc<AtomicIsize>, _wg: GuardWaitGroup) {  
	counter.fetch_add(1, Ordering::SeqCst);  
	// Неявный drop(_wg) вызовет _wg.done(), который декрементирует счетчик
}  
  
fn main() {  
	let counter = Arc::new(AtomicIsize::new(0));
	let wg = GuardWaitGroup::new();  
	for _ in 0..100 {  
	        let wg = wg.clone(); // Вызов wg.add(1)
		let counter = Arc::clone(&counter);  
		thread::spawn(move || process_counter(counter, wg));  
	}  
	wg.wait();  
	println!("{}", counter.load(Ordering::SeqCst)); //100  
}
```

`GuardWaitGroup`, как и `ManualWaitGroup`, хранит в себе `WaitGroupImpl` и делегирует вызовы его методов. Однако, теперь только метод `wait()` является публичным: 

```Rust
pub struct GuardWaitGroup {  
	inner: Arc<WaitGroupImpl>,  
}
impl GuardWaitGroup {
	//...
	pub fn wait(&self) {  
		self.inner.wait();  
	}
	
	fn done(&self) {  
		self.inner.done();  
	}
	
	fn add(&self, delta: usize) {
		self.inner.add(delta);
	}
}
```
Кроме того, для `GuardWaitGroup`  реализованы трейты `Clone` и `Drop`.

Метод `add()` вызывается только внутри `Clone`, инкрементируя счетчик строго на единицу:
```Rust
impl Clone for GuardWaitGroup {  
	fn clone(&self) -> Self {  
		let wg = GuardWaitGroup {  
			inner: Arc::clone(&self.inner),  
		};  
		wg.add(1);  
		wg  
	}  
}  
```

А метод `done()` только при вызове деструктора `Drop`.  Условие здесь необходимо, чтобы мы не вызывали `done()` (и не получали панику), когда вызовется `drop` у "главного", т.е. последнего экземпляра GuardWaitGroup, который в нашем примере в основном потоке ожидает остальных.
```Rust
impl Drop for GuardWaitGroup {  
	fn drop(&mut self) {  
	        if let None = Arc::get_mut(&mut self.inner) {  
			self.done();  
		}  
	}  
}
```

## Преимущества
- Сокращение количества используемого кода. Не нужно явно делать `add` и `done`.
- Теперь мы не можем произвольно менять внутренний счетчик, вроде `add(100)`  или `add(-50)`. Более того, мы *вообще* не можем руками вызывать `done()` и `add()`. Bug 1.1 и 1.2 теперь невозможны.
- Мы не можем "забыть" вызвать `clone()` или `drop()`. 
- Мы гарантированно неявно вызовем `done()` ровно столько же раз, сколько и `add(1)`. Bug 1.3 теперь невозможен
- Мы никак не можем получить панику.

## Недостатки
- Мы не можем произвольно менять счетчик.
- Все еще можем получить дедлок на нетривиальной логике
- В реализации `drop()` для `GuardWaitGroup` Нужно гарантировать отсутствие вызова `done()` при вызове`drop()` на последнем экземпляре, что выглядит несколько костыльно.
- Нет разделения на экземпляры `GuardWaitGroup` которые выполняют логику с изменением счетчика и на экземпляры, которые делают вызов `wait()`.
- Мы практически не можем  сделать корректный вызов `wait()` более одного раза.

	### Bug 2.1 Вызов `wait()` не в том треде
	В приведенном примере у нас есть логическое разделение на экземпляры GuardWaitGroup, которые должны делать RAII-логику c изменением счетчика, и на экземпляр, который должен делать `wait()`. Однако, текущий интерфейс позволяет нам вызвать `wait()` из "вспомогательных" тредов. В общем случае, это вызовет дедлок, поскольку на момент вызова `wait()` счетчик будет всегда больше 0, как минимум равен 1.
	```Rust
	fn some_condition() -> bool { 
		//...
	}
	  
	fn process_counter(counter: Arc<AtomicIsize>, wg: GuardWaitGroup) {  
		counter.fetch_add(1, Ordering::SeqCst);  
		if some_condition() {  
		        wg.wait(); // Дедлок, счетчик всегда больше нуля, потому что мы сделали +1 при вызове clone() но еще не вызвали drop(wg)
		}  
		// Неявный drop(wg) вызовет wg.done()
	}  
	  
	fn main() {  
		let counter = Arc::new(AtomicIsize::new(0));  
		let wg = GuardWaitGroup::new();  
		for _ in 0..100 {  
		        let wg = wg.clone();  
			let counter = Arc::clone(&counter);  
			thread::spawn(move || process_counter(counter, wg));  
		} 
	  
	  wg.wait();  
	}
	```

	<br />

- Более того,  мы не можем это сделать даже если это нужно. Допустим, кроме наших 100 потоков с `process_counter` есть еще `heavy_process_counter` который делает какие-то тяжеловесные вычисления переменной delta, ждет завершения предыдущих потоков, и пишет в `stderr` результат `delta+counter`.
```Rust
fn process_counter(counter: Arc<AtomicIsize>, _wg: GuardWaitGroup) {  
	counter.fetch_add(1, Ordering::SeqCst);  
	// Неявный drop(_wg) вызовет _wg.done()  
}  
  
fn heavy_process_counter(counter: Arc<AtomicIsize>, wg: GuardWaitGroup) {  
	let delta = {  
		sleep(Duration::from_secs(1)); // Эмуляция тяжеловесных вычислений 
		42  
	};  
	wg.wait(); // Дедлок
	eprintln!("{}", counter.load(Ordering::SeqCst) + delta)
	// Неявный drop(_wg) вызовет _wg.done()  
}  
  
fn main() {  
	let counter = Arc::new(AtomicIsize::new(0));  
	let wg = GuardWaitGroup::new();    
	for _ in 0..100 {  
	        let wg = wg.clone();  
		let counter = Arc::clone(&counter);  
		thread::spawn(move || process_counter(counter, wg));  
	}  
	
	{  
		// Спавним еще один поток с дополнительной логикой
		let wg = wg.clone();  
		let counter = Arc::clone(&counter);  
		thread::spawn(move || heavy_process_counter(counter, wg));  
	}
	
	wg.wait();  
	println!("{}", counter.load(Ordering::SeqCst));  // 142
}
```
Здесь мы тоже получим  дедлок.

# SmartWaitGroup
## Использование 
Анализируя примеры из Bug 2.1, можно сделать вывод, что нам нужно разделение WaitGroup на две составляющие. Первая, назовем ее `Waiter`, будет уметь вызывать только `wait()` и не обладать RAII-семантикой. Вторая, назовем ее `Doer`, не будет иметь никаких публичных методов, но будет обладать RAII-семантикой, инкрементируя счетчик в конструкторе, и декрементируя в деструкторе.
Назовем структуру, которая может выдавать такие составляющие, [SmartWaitGroup](https://github.com/Apostoln/WaitGroup/blob/master/src/smart_wait_group.rs).

C ее помощью можно переписать изначальный пример вот так:

```Rust
use wait_group::{Doer, SmartWaitGroup};  
  
fn process_counter(counter: Arc<AtomicIsize>, _doer: Doer) {  
	counter.fetch_add(1, Ordering::SeqCst);  
	// Неявный drop(_wg) вызовет _wg.done()  
}  
  
fn main() {  
	let counter = Arc::new(AtomicIsize::new(0));  
	let wg = SmartWaitGroup::new();  
	for _ in 0..100 {  
	        let doer = wg.doer(); // инкрементируем счетчик 
		let counter = Arc::clone(&counter);  
		thread::spawn(move || process_counter(counter, doer));  
	}  
	wg.waiter().wait();  
	println!("{}", counter.load(Ordering::SeqCst)); //100  
}
```

Здесь мы разделяем SmartWaitGroup на составляющие, отправляя экземпляры Doer в спавнящиеся потоки, и оставляя экземпляр Waiter основном потоке.

Перепишем это еще более явно. Не будем клонировать `SmartWaitGroup` и вызывать руками `waiter()` и `doer()` . Вместо этого, сразу разделим его на `Waiter` и `Doer`:
```Rust
use wait_group::{Doer, SmartWaitGroup};  
  
fn process_counter(counter: Arc<AtomicIsize>, _doer: Doer) {  
	counter.fetch_add(1, Ordering::SeqCst);  
	// Неявный drop(_doer) вызовет _doer.done()  
}  
  
fn spawn_threads(counter: Arc<AtomicIsize>, doer: Doer) {  
	for _ in 0..100 {  
		let doer = doer.clone();  
		let counter = Arc::clone(&counter);  
		thread::spawn(move || process_counter(counter, doer));  
	}  
}  
  
fn main() {  
	let counter = Arc::new(AtomicIsize::new(0));  
	let (waiter, doer) = SmartWaitGroup::splitted();  
	spawn_threads(Arc::clone(&counter), doer);  
	waiter.wait();  
	println!("{}", counter.load(Ordering::SeqCst)); //100  
}
```
Здесь мы вообще не используем `SmartWaitGroup`. Конструктор `splitted()` создаст `SmartWaitGroup`и тут же разделит ее на составляющие, вернув кортеж `(Waiter, Doer)`.

## Реализация  SmartWaitGroup
Как и предыдущие интерфейсы, `SmartWaitGroup` хранит в себе `WaitGroupImpl`.
```Rust
#[derive(Clone)]  
pub struct SmartWaitGroup {  
    inner: Arc<WaitGroupImpl>,  
}
```
Его методы возвращают "обвертки" Waiter и Doer:
```Rust
pub fn waiter(&self) -> Waiter {  
    Waiter::new(Arc::clone(&self.inner))  
}

pub fn doer(&self) -> Doer {  
    Doer::new(Arc::clone(&self.inner))  
}
```

Обвертки также хранят в себе `Arc<WaitGroupImpl>`, таким образом все их методы затрагивают одну и ту же `WaitGroupImpl`.

Doer не имеет публичных методов, но обладает RAII-семантикой, инкрементируя счетчик в конструкторе `new()` (и в `clone()`, соответственно) и декрементируя в деструкторе `drop()`: 
```Rust
#[must_use]  
pub struct Doer {  
	wait_group: Arc<WaitGroupImpl>,  
}  
impl Doer {  
	fn new(wait_group: Arc<WaitGroupImpl>) -> Self {  
		wait_group.increment();  
		Doer { wait_group }  
	}  
  
	fn done(&self) {  
		self.wait_group.done();  
	}  
}  
  
impl Drop for Doer {  
	fn drop(&mut self) {  
		self.done();  
	}  
}  
  
impl Clone for Doer {  
	fn clone(&self) -> Self {  
		Doer::new(Arc::clone(&self.wait_group))  
	}  
}  
```
<br />

`Waiter` имеет единственный метод `wait()`, а его конструкторы-деструкторы тривиальны.
```Rust  
#[must_use]  
pub struct Waiter {  
	wait_group: Arc<WaitGroupImpl>,  
}  
impl Waiter {  
	fn new(wait_group: Arc<WaitGroupImpl>) -> Self {  
	        Waiter { wait_group }  
	}  
  
	pub fn wait(&self) {  
	        self.wait_group.wait();  
	}  
}  
  
impl Clone for Waiter {  
	fn clone(&self) -> Self {  
	        Waiter::new(Arc::clone(&self.wait_group))  
	}  
}
```

Сам `SmartWaitGroup` имеет еще метод `split()` и конструктор `splitted()`, который мы уже видели в примере.
```Rust
pub fn splitted() -> (Waiter, Doer) {  
	Self::new().split()  
}  
  
pub fn split(self) -> (Waiter, Doer) {  
	(self.waiter(), self.doer())  
}
```

## Достоинства
- Все достоинства `GuardWaitGroup`
- Разделение на `Waiter` и `Doer` не позволяет вызвать `wait()` и `add()/done()` на неверных экземплярах.
- Мы теперь не можем написать код, который вызовет Bug 2.1.
- Можно вызывать `wait()` несколько раз без дедлоков. Перепишем второй пример из Bug 2.1.
```Rust
use wait_group::{Doer, Waiter, SmartWaitGroup};  
use std::thread::sleep;  
use std::time::Duration;  
  
fn process_counter(counter: Arc<AtomicIsize>, _doer: Doer) {  
	counter.fetch_add(1, Ordering::SeqCst);  
	// Неявный drop(_doer) вызовет _doer.done()  
}  
  
fn heavy_process_counter(counter: Arc<AtomicIsize>, wg: Waiter) {  
	let delta = {  
	        sleep(Duration::from_secs(1)); // Эмуляция тяжеловесных вычислений
		42  
	};  
	wg.wait(); //Нет дедлока
	eprintln!("{}", counter.load(Ordering::SeqCst) + delta); // 142
}  
  
fn spawn_threads(counter: Arc<AtomicIsize>, doer: Doer) {  
	for _ in 0..100 {  
	        let doer = doer.clone();  
		let counter = Arc::clone(&counter);  
		thread::spawn(move || process_counter(counter, doer));  
	}  
}  

fn spawn_heavy_process_thread(counter: Arc<AtomicIsize>, waiter: Waiter) {  
	let counter = Arc::clone(&counter);  
	thread::spawn(move || heavy_process_counter(counter, waiter));  
}
  
fn main() {  
	let counter = Arc::new(AtomicIsize::new(0));  
	let (waiter, doer) = SmartWaitGroup::splitted();  
	spawn_process_threads(Arc::clone(&counter), doer);  
	spawn_heavy_process_thread(Arc::clone(&counter), waiter.clone());
	waiter.wait();  
	println!("{}", counter.load(Ordering::SeqCst)); //100  
}
```

## Недостатки
- Не всегда возможно разделить программу на потоки, которые должны работать с `Waiter`, и на потоки, которые должны работать c `Doer`. Иногда один поток должен уметь некоторым образом работать с обеими, в таком случае прийдется использовать "цельную" `SmartWaitGroup`, без разделения с помощью `split()`.

# Почему не X?
Возможно, у читателя возникает вопрос, зачем это все нужно, если уже есть замечательный крейт X.


## Почему не [CastellaFactory/wait_group](https://docs.rs/crate/wait_group/0.1.4)?

К сожалению, это крейт предоставляет только непосредственный порт гошных `WaitGroup`, и обладает всеми уже описанными недостками `ManualWaitGroup`. К тому же, `ManualWaitGroup` предоставляет некоторый дополнительные функционал.

## Почему не [crossbeam::sync::WaitGroup](https://docs.rs/crossbeam/0.7.3/crossbeam/sync/struct.WaitGroup.html)?
Эти WaitGroup похожи на `GuardWaitGroup`, но обладают одним критическим недостатком. Выглядит он так:
```rust
pub fn wait(self);
```
Метод `wait()` консьюмит `self`, что делает `WaitGroup` "одноразовой", не позволяя использовать более одного раза, например, использовать как поле структуры  или вызывать `wait()` в двух разных местах.

## Почему не  [crossbeam::scope](https://docs.rs/crossbeam/0.7.3/crossbeam/fn.scope.html) или [rayon::scope](https://docs.rs/rayon/1.3.0/rayon/fn.scope.html)?
Они идеально подходят для случая, когда главный поток должен ждать завершения "вспомогательных" внутри скоупа или тредпула, но случаев использования применения `WaitGroup` намного больше. Рассмотрим их далее.



# Сложный пример
Ебать колотить как я заебался это писать


# Нерешенные проблемы
- Как красиво назвать все эти структуры и методы?
- Как сделать WaitGroupImpl без использования Mutex и Condvar, т.е. без обращений к ОС?
- MultiWaitGroup и прочие варианты удобной работы с несколькими WaitGroup одновременно
- Тесты. Как придумать юзкейсы для тестов?