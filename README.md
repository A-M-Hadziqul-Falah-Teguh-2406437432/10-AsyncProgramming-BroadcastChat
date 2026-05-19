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
