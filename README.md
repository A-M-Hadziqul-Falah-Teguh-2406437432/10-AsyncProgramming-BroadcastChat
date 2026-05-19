# Chat Async - Broadcast Chat dengan Pemrograman Asinkron

## Refleksi Experiment 2.1: Original code, and how it run

![Experiment 2.1](images/Experiment2.1.jpg)

Pada eksperimen ini, saya menjalankan aplikasi broadcast chat yang menggunakan pemrograman asinkron dengan menjalankan satu unit server dan tiga unit client secara bersamaan.

Untuk memulai aplikasi, saya menjalankan perintah `cargo run --bin server` pada satu terminal untuk membuka listener pada port 2000, kemudian menjalankan `cargo run --bin client` di tiga terminal lainnya untuk menghubungkan mereka ke server.

Ketika sebuah pesan diketik pada salah satu client, pesan tersebut dikirimkan ke server melalui protokol websocket secara asinkron tanpa memblokir input dari pengguna lain.

Server kemudian menerima pesan tersebut dan menyiarkannya (broadcast) kembali ke seluruh client yang terhubung sehingga semua pengguna dapat melihat pesan yang dikirimkan.

Hal ini menunjukkan efisiensi pemrograman asinkron dalam menangani banyak koneksi I/O-bound sekaligus hanya dengan menggunakan sumber daya CPU yang minimal dibandingkan jika menggunakan OS threads tradisional.

Melalui percobaan ini, terlihat bahwa server mampu mengelola status banyak koneksi secara konkuren, memproses pesan masuk, dan mendistribusikannya secara real-time.

Penggunaan protokol websocket di sini sangat krusial karena memungkinkan komunikasi dua arah yang persisten dan lebih efisien daripada siklus request-response HTTP biasa.

## Refleksi Experiment 2.2: Modifying port

Dalam eksperimen ini, saya berhasil mengubah port komunikasi websocket dari port default 2000 menjadi port 8080 untuk meningkatkan pemahaman mengenai konfigurasi jaringan pada aplikasi asinkron.

Modifikasi ini harus dilakukan pada dua sisi utama, yaitu pada file `src/bin/server.rs` di bagian `TcpListener::bind` dan pada file `src/bin/client.rs` di bagian pendefinisian URI websocket.

Perubahan pada kedua sisi sangat krusial karena websocket merupakan protokol berbasis koneksi; jika port pada sisi listener (server) dan connector (client) tidak sama, maka jabat tangan (handshake) awal akan gagal dan koneksi tidak dapat terjalin.

Meskipun port diubah, aplikasi ini tetap menggunakan protokol websocket (`ws://`) yang sama, yang menyediakan saluran komunikasi dua arah yang persisten di atas koneksi TCP tunggal.

Definisi protokol ini secara teknis diatur melalui crate `tokio_websockets` yang digunakan dalam kode, di mana crate tersebut menangani proses enkapsulasi pesan ke dalam frame websocket agar dapat dibaca oleh kedua belah pihak.

Selain file server dan client utama, tidak ada file konfigurasi lain yang perlu diubah karena alamat IP dan port didefinisikan secara langsung (hardcoded) di dalam kode sumber tersebut.

Setelah menjalankan kembali sistem dengan port 8080, saya memverifikasi bahwa fungsionalitas broadcast tetap berjalan sempurna, di mana pesan dari satu client tetap dapat diterima oleh server dan disiarkan ke seluruh client lainnya secara asinkron.

## Experiment 2.3: Small changes, add IP and Port

![Screenshot Experiment 2.3](images/Experiment2.3.jpg)

1. Pada eksperimen ini, saya memodifikasi logika server agar setiap pesan yang disiarkan ke seluruh client menyertakan identitas unik berupa alamat IP dan nomor port pengirim.
2. Informasi alamat ini diperoleh dari variabel `addr` bertipe `SocketAddr` yang ditangkap oleh server saat proses jabat tangan TCP pertama kali terjadi.
3. Modifikasi dilakukan dengan menggunakan makro `format!` di sisi server untuk menggabungkan alamat pengirim dengan isi pesan teks sebelum dikirim ke saluran broadcast.
4. Perubahan ini memungkinkan setiap pengguna chat untuk mengidentifikasi asal pesan secara spesifik meskipun aplikasi belum memiliki sistem login atau username.
5. Di sisi client, saya menyesuaikan output terminal agar pesan yang diterima dari server ditampilkan dengan format yang lebih menonjol agar informasi pengirim mudah dibaca.
6. Hal ini menunjukkan bagaimana metadata koneksi dalam pemrograman asinkron dapat dimanfaatkan untuk memperkaya data aplikasi tanpa memerlukan input tambahan dari pengguna.
7. Dengan menyertakan port, kita bisa membedakan antar client meskipun mereka berasal dari IP lokal yang sama (127.0.0.1), karena setiap koneksi websocket akan membuka port ephemeral yang berbeda di sisi client.

## Bonus: Rust Websocket server for YewChat!

Untuk tantangan bonus ini, saya memodifikasi server Tutorial 2 agar mampu melayani webchat YewChat dari Tutorial 3, menggantikan peran server JavaScript (`SimpleWebsocketServer`) yang berbasis `ws` dan TypeScript. Implementasi baru saya tempatkan pada binary tambahan `src/bin/yew_server.rs` agar binary `server.rs` dan `client.rs` yang asli untuk eksperimen 2.1 – 2.3 tetap dapat dijalankan tanpa perubahan. Server baru dijalankan dengan perintah `cargo run --bin yew_server` dan, sama seperti versi JavaScript, mendengarkan pada `127.0.0.1:8080` sehingga klien Yew tidak perlu mengubah URL koneksinya sama sekali.

### Bagaimana cara saya melakukannya

Perbedaan inti antara server Tutorial 2 dengan server JavaScript Tutorial 3 adalah pada *protokol di atas WebSocket*. Server Tutorial 2 hanya menerima teks bebas lalu menyiarkannya kembali dengan awalan `[addr]:`, sedangkan klien YewChat selalu mengirim dan mengharapkan pesan dalam format JSON dengan amplop seperti `{"messageType":"register","data":"<nick>"}`, `{"messageType":"message","data":"<text>"}`, dan menerima balasan `{"messageType":"users","dataArray":[...]}` serta `{"messageType":"message","data":"<inner-json>"}` yang `data`-nya kembali berupa JSON terserialisasi berisi `from`, `message`, dan `time`. Untuk menjembatani ini saya menambahkan dependensi `serde` dan `serde_json` pada `Cargo.toml`, lalu mendefinisikan dua struct: `WsMessage` dengan `#[serde(rename_all = "camelCase")]` agar serialisasinya cocok dengan format JS (`messageType`, `dataArray`), dan `ChatPayload` untuk isi `data` pada pesan chat.

Kerangka asinkronnya tetap mempertahankan pola Tutorial 2 — `TcpListener::bind`, `tokio::spawn` per koneksi, dan `tokio::sync::broadcast::channel` untuk fan-out — tetapi state daftar pengguna saya simpan pada `Arc<Mutex<HashMap<SocketAddr, String>>>` (memakai `tokio::sync::Mutex` agar aman dipegang melewati `await`) yang di-`clone` ke setiap task koneksi. Pada setiap frame masuk, server mem-parsing JSON-nya, mencocokkan `message_type`, dan kemudian: (1) untuk `register`, menyimpan pemetaan `addr → nick` lalu memanggil `broadcast_users` yang menyiarkan envelope `users` berisi seluruh nickname aktif; (2) untuk `message`, mencari nickname pengirim dari peta tersebut, membungkusnya ke dalam `ChatPayload` dengan timestamp `SystemTime::now().duration_since(UNIX_EPOCH)`, lalu mengirim envelope `message` dengan `data` berupa string JSON dari payload tersebut. Saat klien diskonek (loop `ws_stream.next()` mengembalikan `None`), entrinya dihapus dari peta dan daftar `users` kembali disiarkan agar sidebar pada YewChat ikut ter-update secara real-time.

### Mengapa perubahan ini berhasil

Perubahan ini saya anggap berhasil karena tidak diperlukan modifikasi apa pun pada sisi klien Yew: file `services/websocket.rs` tetap membuka `ws://127.0.0.1:8080`, file `components/chat.rs` tetap men-deserialisasi `WebSocketMessage` dengan field `messageType`, `data`, dan `dataArray` yang sama persis, dan logika nested-JSON pada pesan chat (`serde_json::from_str(&msg.data.unwrap())` menjadi `MessageData { from, message }`) tetap valid karena server baru menulis dengan struktur yang identik. Saya melakukan verifikasi dengan menjalankan `cargo check --bin yew_server` (lulus tanpa warning) dan menjalankan server bersama klien YewChat: login dengan dua tab username berbeda menyebabkan sidebar `Users` di kedua tab menampilkan kedua nickname, dan setiap pesan yang dikirim langsung muncul di kedua tab dengan nama pengirim yang benar serta avatar DiceBear yang sesuai. Sinkronisasi disconnect juga ter-uji: menutup satu tab membuat tab lainnya kehilangan entry user tersebut dalam waktu satu trip broadcast.

### Pendapat: JavaScript vs Rust

Setelah mengimplementasikan kedua versi, saya pribadi lebih memilih versi **Rust**, walau saya menghargai versi JavaScript-nya. Versi JavaScript memang lebih ringkas — sekitar 60 baris dengan tipe dinamis dan callback `ws.on('message', ...)` yang langsung mengakses field JSON tanpa deklarasi tipe — sehingga untuk prototyping cepat ia memang lebih nyaman. Namun versi Rust memberi tiga keunggulan yang saya rasakan langsung: pertama, *type-safety* lewat `serde` membuat kesalahan seperti salah nama field (`messageType` vs `message_type`) tertangkap saat kompilasi alih-alih jadi bug runtime, sehingga refactor lebih percaya diri; kedua, *concurrency model*-nya eksplisit — `Arc<Mutex<…>>` dan `broadcast::channel` memaksa saya memikirkan kepemilikan dan visibility data antar task, yang pada server JavaScript justru tersembunyi di balik event-loop dan rentan bug seperti `users` list yang dimodifikasi sambil disiarkan; ketiga, performa dan penggunaan memori-nya jauh lebih efisien karena `tokio` menjadwalkan ribuan task asinkron di atas thread pool tanpa overhead V8, yang relevan ketika jumlah klien membesar. Trade-off-nya, kompilasi Rust lebih lambat dan kurva belajarnya lebih curam, sehingga untuk eksperimen kecil seperti tutorial ini versi JavaScript bisa dibilang "cukup baik" — tetapi untuk basis kode yang akan tumbuh, versi Rust jelas lebih layak dipertahankan.
