# Connection Storage System

## Overview
Sistem ini secara otomatis menyimpan semua koneksi database ke dalam satu file JSON terenkripsi.

## Fitur Keamanan

### Enkripsi AES-256-GCM
- **Algorithm**: AES-256-GCM (Galois/Counter Mode)
- **Key Size**: 256 bits
- **Authentication**: Built-in message authentication

### Key Derivation
- Password dibuat dari kombinasi hostname mesin
- Salt: `rustdbgrid_v1_salt_2025`
- Hash: SHA-256

### Perlindungan Password
- Password database dienkripsi sebelum disimpan
- Setiap password memiliki nonce unik (12 bytes)
- Data dienkode dengan Base64

## Storage Location

### Windows
```
C:\Users\<username>\AppData\Roaming\rustdbgrid\connections.json
```

### Linux
```
~/.config/rustdbgrid/connections.json
```

### macOS
```
~/Library/Application Support/rustdbgrid/connections.json
```

## Auto-Save Behavior

### Kapan File Disimpan?
1. **Saat menambah koneksi baru** - Otomatis tersimpan setelah klik "Save"
2. **Saat mengupdate koneksi** - Otomatis tersimpan saat edit selesai
3. **Saat menghapus koneksi** - Otomatis tersimpan setelah delete

### Tidak Perlu Button Trigger
- ✅ Semua operasi otomatis save ke file
- ✅ Tidak perlu tombol "Save All" atau "Export"
- ✅ Data persisten antar restart aplikasi

## File Format

### Structure (Encrypted)
```json
{
  "connections": [
    {
      "id": "uuid-here",
      "name": "Production DB",
      "db_type": "PostgreSQL",
      "host": "localhost",
      "port": 5432,
      "username": "admin",
      "password_encrypted": "base64_encrypted_data_here",
      "database": "mydb",
      "ssl": true
    }
  ]
}
```

### Password Encryption Process
1. Original password: `my_secret_pass`
2. Encryption password: `rustdbgrid_<hostname>`
3. Salt: `rustdbgrid_v1_salt_2025`
4. Key derivation: SHA256(password + salt)
5. Encrypt: AES-256-GCM(data, key, nonce)
6. Output: Base64(nonce + ciphertext)

## Security Notes

### ⚠️ Important
- File terenkripsi dengan key berbeda per mesin
- Jika pindah mesin, koneksi tidak bisa didekripsi
- Backup file connections.json secara terpisah jika diperlukan

### 🔒 Best Practices
1. Jangan share file connections.json ke orang lain
2. Jangan commit file ini ke git repository
3. Gunakan OS-level encryption untuk extra security
4. Backup regular untuk disaster recovery

## Technical Details

### Dependencies
- `aes-gcm 0.10` - Encryption
- `sha2 0.10` - Key derivation
- `base64 0.22` - Encoding
- `rand 0.8` - Nonce generation
- `dirs 5.0` - Config directory
- `hostname 0.4` - Machine identification

### Error Handling
- Jika file corrupt, aplikasi akan start dengan koneksi kosong
- Jika dekripsi gagal, koneksi yang gagal akan di-skip
- Log error akan ditampilkan di console

## Migration Guide

### Dari Versi Sebelumnya
Jika sebelumnya koneksi tersimpan di memory saja:
1. Launch aplikasi versi baru
2. Koneksi lama akan hilang (tidak ada migration)
3. Tambah ulang koneksi, akan otomatis tersimpan

### Export/Import (Future Feature)
Planned features:
- Export koneksi ke file terpisah
- Import koneksi dari file backup
- Sync antar devices dengan cloud
