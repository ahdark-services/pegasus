package action_reply

import (
	"github.com/mymmrac/telego"
	"github.com/stretchr/testify/assert"
	"testing"
)

func TestCheckIfChinese(t *testing.T) {
	type args struct {
		r rune
	}

	tests := []struct {
		name string
		args args
		want bool
	}{
		{
			name: "Chinese",
			args: args{r: '你'},
			want: true,
		},
		{
			name: "Not Chinese",
			args: args{r: 'a'},
			want: false,
		},
	}

	asserts := assert.New(t)
	for _, tt := range tests {
		asserts.Equal(tt.want, checkIfChinese(tt.args.r), tt.name)
	}
}

func TestGetFullUserNickname(t *testing.T) {
	type args struct {
		user *telego.User
	}

	tests := []struct {
		name string
		args args
		want string
	}{
		{
			name: "With last name",
			args: args{user: &telego.User{FirstName: "John", LastName: "Doe"}},
			want: "John Doe",
		},
		{
			name: "Without last name",
			args: args{user: &telego.User{FirstName: "John", LastName: ""}},
			want: "John",
		},
	}

	asserts := assert.New(t)
	for _, tt := range tests {
		asserts.Equal(tt.want, getFullUserNickname(tt.args.user), tt.name)
	}
}
