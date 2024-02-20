package cryptor

import (
	"crypto/rand"
	"github.com/stretchr/testify/assert"
	"testing"
)

func TestBase64StdEncode(t *testing.T) {
	asserts := assert.New(t)
	asserts.Equal("aGVsbG8gd29ybGQ=", string(Base64StdEncode([]byte("hello world"))))
}

func BenchmarkBase64StdEncode(b *testing.B) {
	testData := []byte("7z0lecZIWP!!Nw#-i$2^@rukXwWyYN_NyGMSrY-3$hjlFpBx==mJ=W9o$5JkjHDY")

	b.StartTimer()
	for i := 0; i < b.N; i++ {
		Base64StdEncode(testData)
	}
	b.StopTimer()
}

func TestBase64StdDecode(t *testing.T) {
	asserts := assert.New(t)
	data, err := Base64StdDecode([]byte("aGVsbG8gd29ybGQ="))
	asserts.NoError(err)
	asserts.Equal("hello world", string(data))
}

func BenchmarkBase64StdDecode(b *testing.B) {
	testData := []byte("N3owbGVjWklXUCEhTncjLWkkMl5AcnVrWHdXeVlOX055R01TclktMyRoamxGcEJ4PT1tSj1XOW8kNUprakhEWQ==")

	b.StartTimer()
	for i := 0; i < b.N; i++ {
		Base64StdDecode(testData)
	}
	b.StopTimer()
}

func TestMd5(t *testing.T) {
	asserts := assert.New(t)
	asserts.Equal("5d41402abc4b2a76b9719d911017c592", string(Md5([]byte("hello"))))
}

func BenchmarkMd5(b *testing.B) {
	b.Run("1KB", func(b *testing.B) {
		test := make([]byte, 1024)
		rand.Read(test)

		b.StartTimer()
		for i := 0; i < b.N; i++ {
			Md5(test)
		}
		b.StopTimer()
	})

	b.Run("32KB", func(b *testing.B) {
		test := make([]byte, 1024*32)
		rand.Read(test)

		b.StartTimer()
		for i := 0; i < b.N; i++ {
			Md5(test)
		}
		b.StopTimer()
	})
}

func TestMd5WithBase64(t *testing.T) {
	asserts := assert.New(t)
	asserts.Equal("XUFAKrxLKna5cZ2REBfFkg==", string(Md5WithBase64([]byte("hello"))))
}

func TestMd5File(t *testing.T) {
	asserts := assert.New(t)
	fileMd5, err := Md5File("./basic.go")
	asserts.NoError(err)
	asserts.NotNil(fileMd5)
}

func TestHmacMd5(t *testing.T) {
	asserts := assert.New(t)
	asserts.Equal("5f4c9faaff0a1ad3007d9ddc06abe36d", string(HmacMd5([]byte("hello world"), []byte("12345"))))
}

func TestHmacMd5WithBase64(t *testing.T) {
	asserts := assert.New(t)
	asserts.Equal("6DQwbquJLYclJdSRinpjmg==", string(HmacMd5WithBase64([]byte("hello"), []byte("12345"))))
}

func TestHmacSha1(t *testing.T) {
	asserts := assert.New(t)
	asserts.Equal("3826f812255d8683f051ee97346d1359234d5dbd", string(HmacSha1([]byte("hello world"), []byte("12345"))))
}

func TestHmacSha1WithBase64(t *testing.T) {
	asserts := assert.New(t)
	asserts.Equal("XGqdsMzLkuNu0DI/0Jt/k23prOA=", string(HmacSha1WithBase64([]byte("hello"), []byte("12345"))))
}

func TestHmacSha256(t *testing.T) {
	asserts := assert.New(t)
	asserts.Equal("9dce2609f2d67d41f74c7f9efc8ccd44370d41ad2de52982627588dfe7289ab8", string(HmacSha256([]byte("hello world"), []byte("12345"))))
}

func BenchmarkHmacSha256(b *testing.B) {
	b.Run("1KB", func(b *testing.B) {
		data := make([]byte, 1024)
		rand.Read(data)

		key := make([]byte, 32)
		rand.Read(key)

		b.StartTimer()
		for i := 0; i < b.N; i++ {
			HmacSha256(data, key)
		}
		b.StopTimer()
	})

	b.Run("32KB", func(b *testing.B) {
		data := make([]byte, 1024*32)
		rand.Read(data)

		key := make([]byte, 32)
		rand.Read(key)

		b.StartTimer()
		for i := 0; i < b.N; i++ {
			HmacSha256(data, key)
		}
		b.StopTimer()
	})
}

func TestHmacSha256WithBase64(t *testing.T) {
	asserts := assert.New(t)
	asserts.Equal("MVu5PE6YmGK6Ccti4F1zpfN2yzbw14btqwwyDQWf3nU=", string(HmacSha256WithBase64([]byte("hello"), []byte("12345"))))
}

func TestHmacSha512(t *testing.T) {
	asserts := assert.New(t)
	asserts.Equal("5b1563ac4e9b49c9ada8ccb232588fc4f0c30fd12f756b3a0b95af4985c236ca60925253bae10ce2c6bf9af1c1679b51e5395ff3d2826c0a2c7c0d72225d4175", string(HmacSha512([]byte("hello world"), []byte("12345"))))
}

func TestHmacSha512WithBase64(t *testing.T) {
	asserts := assert.New(t)
	asserts.Equal("3Y8SkKndI9NU4lJtmi6c6M///dN8syCADRxsE9Lvw2Mog3ahlsVFja9T+OGqa0Wm2FYwPVwKIGS/+XhYYdSM/A==", string(HmacSha512WithBase64([]byte("hello"), []byte("12345"))))
}

func TestSha1(t *testing.T) {
	asserts := assert.New(t)
	asserts.Equal("2aae6c35c94fcfb415dbe95f408b9ce91ee846ed", string(Sha1([]byte("hello world"))))
}

func TestSha1WithBase64(t *testing.T) {
	asserts := assert.New(t)
	asserts.Equal("qvTGHdzF6KLavt4PO0gs2a6pQ00=", string(Sha1WithBase64([]byte("hello"))))
}

func TestSha256(t *testing.T) {
	asserts := assert.New(t)
	asserts.Equal("b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9", string(Sha256([]byte("hello world"))))
}

func BenchmarkSha256(b *testing.B) {
	b.Run("1KB", func(b *testing.B) {
		data := make([]byte, 1024)
		rand.Read(data)

		b.StartTimer()
		for i := 0; i < b.N; i++ {
			Sha256(data)
		}
		b.StopTimer()
	})

	b.Run("32KB", func(b *testing.B) {
		data := make([]byte, 1024*32)
		rand.Read(data)

		b.StartTimer()
		for i := 0; i < b.N; i++ {
			Sha256(data)
		}
		b.StopTimer()
	})
}

func TestSha256WithBase64(t *testing.T) {
	asserts := assert.New(t)
	asserts.Equal("LPJNul+wow4m6DsqxbninhsWHlwfp0JecwQzYpOLmCQ=", string(Sha256WithBase64([]byte("hello"))))
}

func TestSha512(t *testing.T) {
	asserts := assert.New(t)
	asserts.Equal("309ecc489c12d6eb4cc40f50c902f2b4d0ed77ee511a7c7a9bcd3ca86d4cd86f989dd35bc5ff499670da34255b45b0cfd830e81f605dcf7dc5542e93ae9cd76f", string(Sha512([]byte("hello world"))))
}

func TestSha512WithBase64(t *testing.T) {
	asserts := assert.New(t)
	asserts.Equal("m3HSJL1i83hdltRq0+o9czGb+8KJDKra4t/3JRlnPKcjI8PZm6XBHXx6zG4UuMXaDEZjR1wuXDre9G9zvN7AQw==", string(Sha512WithBase64([]byte("hello"))))
}
