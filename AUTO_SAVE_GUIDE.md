# Auto-Save Connection System

## 🎯 Fitur Utama

### ✅ Otomatis Tersimpan
- **TIDAK ADA TOMBOL TRIGGER** - Semua koneksi otomatis tersimpan
- **1 FILE JSON** - Semua koneksi disimpan dalam satu file
- **ENKRIPSI AES-256-GCM** - Password dienkripsi dengan salt

## 📁 Lokasi File

### Windows
```
C:\Users\<YourName>\AppData\Roaming\rustdbgrid\connections.json
```

### Linux
```
~/.config/rustdbgrid/connections.json
```

### macOS
```
~/Library/Application Support/rustdbgrid/connections.json
```

## 🔄 Kapan Data Disimpan?

### 1. **Tambah Koneksi Baru**
```
Klik "Add Connection" → Isi form → Klik "Save" 
→ ✅ OTOMATIS tersimpan ke file JSON
```

### 2. **Edit Koneksi**
```
Klik connection → Edit data → Klik "Save"
→ ✅ OTOMATIS update file JSON
```

### 3. **Hapus Koneksi**
```
Klik delete button → Confirm
→ ✅ OTOMATIS hapus dari file JSON
```

## 🔐 Keamanan

### Enkripsi Password
```javascript
// Password ASLI (tidak pernah disimpan plain text)
password: "rahasia123"

// Password TERENKRIPSI dalam file
password_encrypted: "AQIDBAUGBwgJCgsMDQ4P..." // Base64 encoded
```

### Sistem Enkripsi
1. **Algorithm**: AES-256-GCM (Military-grade encryption)
2. **Key Derivation**: SHA-256(password + salt)
3. **Salt**: `rustdbgrid_v1_salt_2025`
4. **Nonce**: Random 12 bytes per encryption
5. **Password Source**: Hostname komputer Anda

### Contoh File JSON (Encrypted)
```json
{
  "connections": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "Production PostgreSQL",
      "db_type": "PostgreSQL",
      "host": "localhost",
      "port": 5432,
      "username": "admin",
      "password_encrypted": "AeXwZ8k9mN4pQ6rS...", // ← ENCRYPTED!
      "database": "myapp",
      "ssl": true
    }
  ]
}
```

## 💡 Cara Kerja

### Alur Penyimpanan
```
1. User klik "Save" di Connection Modal
   ↓
2. Frontend call: saveConnection(config)
   ↓
3. Backend Rust:
   - Encrypt password dengan AES-256-GCM
   - Convert ke StoredConnection
   - Save ke connections.json
   ↓
4. File otomatis dibuat/update
   ↓
5. UI update - tampil di sidebar
```

### Alur Load Connections
```
1. Aplikasi start
   ↓
2. ConnectionStore::new() dipanggil
   ↓
3. Load connections.json dari disk
   ↓
4. Decrypt semua passwords
   ↓
5. Load ke memory
   ↓
6. Tampil di UI sidebar
```

## 🎨 UI Indicators

### Sidebar Footer
```
┌─────────────────────────┐
│ 💾 STORAGE              │
│ ───────────────────────│
│ 📁 Auto-saved          │
│ 🔒 Encrypted (2.3 KB)  │
└─────────────────────────┘
```

### Connection Modal Footer
```
┌──────────────────────────────────┐
│ 🛡️ Connection auto-saved to     │
│    encrypted file                │
└──────────────────────────────────┘
```

## ⚠️ PENTING!

### Jangan Share File Ini
❌ **JANGAN** commit `connections.json` ke Git
❌ **JANGAN** share file ini ke orang lain
❌ **JANGAN** copy ke komputer lain (enkripsi berbeda)

### Backup
✅ Backup `connections.json` secara terpisah
✅ Simpan di tempat aman (encrypted storage)
✅ Gunakan cloud backup dengan encryption

## 🔧 Troubleshooting

### File Tidak Ada
**Q**: Kenapa file connections.json tidak ada?  
**A**: File dibuat otomatis saat Anda save koneksi pertama kali.

### Koneksi Hilang
**Q**: Kenapa koneksi hilang setelah restart?  
**A**: Mungkin file corrupt atau terhapus. Cek di lokasi file.

### Error Dekripsi
**Q**: Error "Decryption failed"?  
**A**: File mungkin dari komputer lain atau corrupt. Hapus file dan buat baru.

### Lokasi File
**Q**: Bagaimana cara lihat lokasi file?  
**A**: Cek di sidebar footer, atau lihat console log saat startup.

## 🧪 Testing

### Test Manual
1. Tambah koneksi baru → Save
2. Close aplikasi
3. Buka aplikasi lagi
4. ✅ Koneksi masih ada

### Verifikasi File
```powershell
# Windows PowerShell
cd $env:APPDATA\rustdbgrid
cat connections.json | ConvertFrom-Json | ConvertTo-Json -Depth 10
```

```bash
# Linux/Mac
cd ~/.config/rustdbgrid
cat connections.json | jq .
```

## 📊 File Size

Estimasi ukuran file:
- **1 koneksi** ≈ 300-500 bytes
- **10 koneksi** ≈ 3-5 KB
- **100 koneksi** ≈ 30-50 KB

File sangat kecil, tidak akan makan space disk!

## 🚀 Future Features

Planned:
- [ ] Export connections ke file backup
- [ ] Import connections dari file
- [ ] Sync antar devices dengan cloud
- [ ] Compression untuk file besar
- [ ] Multiple file profiles

## 💬 Support

Jika ada masalah dengan auto-save:
1. Check console log di DevTools
2. Verify file permissions di config directory
3. Try delete file dan recreate
4. Check disk space

---

**No manual save button needed! 🎉**  
Everything is automatic and encrypted! 🔒
