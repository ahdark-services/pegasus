// Copyright 2021 dudaodong@gmail.com. All rights reserved.
// Use of this source code is governed by MIT license

// Package cryptor implements some util functions to encrypt and decrypt.
// Contain base64, hmac, sha, aes, des, and rsa
package cryptor

import (
	"bufio"
	"crypto/hmac"
	"crypto/md5"
	"crypto/sha1"
	"crypto/sha512"
	"encoding/base64"
	"encoding/hex"
	"fmt"
	"io"
	"os"

	md5simd "github.com/minio/md5-simd"
	sha256simd "github.com/minio/sha256-simd"
)

// Base64StdEncode encode string with base64 encoding.
func Base64StdEncode(s []byte) []byte {
	b := make([]byte, base64.StdEncoding.EncodedLen(len(s)))
	base64.StdEncoding.Encode(b, s)
	return b
}

// Base64StdDecode decode a base64 encoded string.
func Base64StdDecode(s []byte) ([]byte, error) {
	b := make([]byte, base64.StdEncoding.DecodedLen(len(s)))
	n, err := base64.StdEncoding.Decode(b, s)
	if err != nil {
		return nil, err
	}

	return b[:n], nil
}

var md5Server = md5simd.NewServer()

// Md5 return the md5 value of bytes.
func Md5(s []byte) []byte {
	h := md5Server.NewHash()
	defer h.Close()

	h.Write(s)
	dst := make([]byte, 32)
	hex.Encode(dst, h.Sum(nil))
	return dst
}

// Md5WithBase64 return the md5 value of bytes with base64.
func Md5WithBase64(s []byte) []byte {
	h := md5Server.NewHash()
	defer h.Close()

	h.Write(s)
	sum := h.Sum(nil)
	dst := make([]byte, base64.StdEncoding.EncodedLen(len(sum)))
	base64.StdEncoding.Encode(dst, sum)
	return dst
}

// Md5File return the md5 value of file.
func Md5File(filename string) (string, error) {
	if fileInfo, err := os.Stat(filename); err != nil {
		return "", err
	} else if fileInfo.IsDir() {
		return "", nil
	}

	file, err := os.Open(filename)
	if err != nil {
		return "", err
	}
	defer file.Close()

	hash := md5Server.NewHash()
	defer hash.Close()

	chunkSize := 65536
	for buf, reader := make([]byte, chunkSize), bufio.NewReader(file); ; {
		n, err := reader.Read(buf)
		if err != nil {
			if err == io.EOF {
				break
			}
			return "", err
		}
		hash.Write(buf[:n])
	}

	checksum := fmt.Sprintf("%x", hash.Sum(nil))
	return checksum, nil
}

// HmacMd5 return the hmac hash of string use md5.
func HmacMd5(str, key []byte) []byte {
	h := hmac.New(md5.New, key)
	h.Write(str)
	dst := make([]byte, 32)
	hex.Encode(dst, h.Sum(nil))
	return dst
}

// HmacMd5WithBase64 return the hmac hash of string use md5 with base64.
// https://go.dev/play/p/UY0ng2AefFC
func HmacMd5WithBase64(data, key []byte) []byte {
	h := hmac.New(md5.New, key)
	h.Write(data)
	sum := h.Sum(nil)
	dst := make([]byte, base64.StdEncoding.EncodedLen(len(sum)))
	base64.StdEncoding.Encode(dst, sum)
	return dst
}

// HmacSha1 return the hmac hash of string use sha1.
func HmacSha1(str, key []byte) []byte {
	h := hmac.New(sha1.New, key)
	h.Write(str)
	dst := make([]byte, 40)
	hex.Encode(dst, h.Sum(nil))
	return dst
}

// HmacSha1WithBase64 return the hmac hash of string use sha1 with base64.
func HmacSha1WithBase64(str, key []byte) []byte {
	h := hmac.New(sha1.New, key)
	h.Write(str)
	sum := h.Sum(nil)
	dst := make([]byte, base64.StdEncoding.EncodedLen(len(sum)))
	base64.StdEncoding.Encode(dst, sum)
	return dst
}

// HmacSha256 return the hmac hash of string use sha256.
func HmacSha256(str, key []byte) []byte {
	h := hmac.New(sha256simd.New, key)
	h.Write(str)
	dst := make([]byte, 64)
	hex.Encode(dst, h.Sum(nil))
	return dst
}

// HmacSha256WithBase64 return the hmac hash of string use sha256 with base64.
func HmacSha256WithBase64(str, key []byte) []byte {
	h := hmac.New(sha256simd.New, key)
	h.Write(str)
	sum := h.Sum(nil)
	dst := make([]byte, base64.StdEncoding.EncodedLen(len(sum)))
	base64.StdEncoding.Encode(dst, sum)
	return dst
}

// HmacSha512 return the hmac hash of string use sha512.
func HmacSha512(str, key []byte) []byte {
	h := hmac.New(sha512.New, key)
	h.Write(str)
	sum := h.Sum(nil)
	dst := make([]byte, 128)
	hex.Encode(dst, sum)
	return dst
}

// HmacSha512WithBase64 return the hmac hash of string use sha512 with base64.
func HmacSha512WithBase64(str, key []byte) []byte {
	h := hmac.New(sha512.New, key)
	h.Write(str)
	sum := h.Sum(nil)
	dst := make([]byte, base64.StdEncoding.EncodedLen(len(sum)))
	base64.StdEncoding.Encode(dst, sum)
	return dst
}

// Sha1 return the sha1 value (SHA-1 hash algorithm) of string.
func Sha1(str []byte) []byte {
	hash := sha1.New()
	hash.Write(str)
	dst := make([]byte, 40)
	hex.Encode(dst, hash.Sum(nil))
	return dst
}

// Sha1WithBase64 return the sha1 value (SHA-1 hash algorithm) of base64 string.
func Sha1WithBase64(str []byte) []byte {
	hash := sha1.New()
	hash.Write(str)
	sum := hash.Sum(nil)
	dst := make([]byte, base64.StdEncoding.EncodedLen(len(sum)))
	base64.StdEncoding.Encode(dst, sum)
	return dst
}

// Sha256 return the sha256 value (SHA256 hash algorithm) of string.
func Sha256(str []byte) []byte {
	hash := sha256simd.New()
	hash.Write(str)
	dst := make([]byte, 64)
	hex.Encode(dst, hash.Sum(nil))
	return dst
}

// Sha256WithBase64 return the sha256 value (SHA256 hash algorithm) of base64 string.
func Sha256WithBase64(str []byte) []byte {
	hash := sha256simd.New()
	hash.Write(str)
	sum := hash.Sum(nil)
	dst := make([]byte, base64.StdEncoding.EncodedLen(len(sum)))
	base64.StdEncoding.Encode(dst, sum)
	return dst
}

// Sha512 return the sha512 value (SHA512 hash algorithm) of string.
func Sha512(str []byte) []byte {
	hash := sha512.New()
	hash.Write(str)
	dst := make([]byte, 128)
	hex.Encode(dst, hash.Sum(nil))
	return dst
}

// Sha512WithBase64 return the sha512 value (SHA512 hash algorithm) of base64 string.
func Sha512WithBase64(str []byte) []byte {
	hash := sha512.New()
	hash.Write(str)
	sum := hash.Sum(nil)
	dst := make([]byte, base64.StdEncoding.EncodedLen(len(sum)))
	base64.StdEncoding.Encode(dst, sum)
	return dst
}
