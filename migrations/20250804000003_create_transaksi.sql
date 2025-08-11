-- Create transaksi table
CREATE TABLE IF NOT EXISTS transaksi (
    id SERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    kategori_id INTEGER NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
    jumlah INTEGER NOT NULL CHECK (jumlah > 0),
    deskripsi TEXT NOT NULL,
    tanggal DATE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_transaksi_user_id ON transaksi(user_id);
CREATE INDEX IF NOT EXISTS idx_transaksi_kategori_id ON transaksi(kategori_id);
CREATE INDEX IF NOT EXISTS idx_transaksi_tanggal ON transaksi(tanggal);
CREATE INDEX IF NOT EXISTS idx_transaksi_user_tanggal ON transaksi(user_id, tanggal);
CREATE INDEX IF NOT EXISTS idx_transaksi_created_at ON transaksi(created_at);
