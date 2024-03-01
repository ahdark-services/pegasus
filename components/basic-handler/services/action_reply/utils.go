package action_reply

import (
	"fmt"
	"github.com/mymmrac/telego"
)

func checkIfChinese(r rune) bool {
	return (r >= '\u4e00' && r <= '\u9fff') ||
		(r >= '\u3400' && r <= '\u4dbf') ||
		(r >= '\U00020000' && r <= '\U0002A6DF') ||
		(r >= '\U0002A700' && r <= '\U0002B73F') ||
		(r >= '\U0002B740' && r <= '\U0002B81F') ||
		(r >= '\U0002B820' && r <= '\U0002CEAF') ||
		(r >= '\U0002CEB0' && r <= '\U0002EBEF') ||
		(r >= '\U00030000' && r <= '\U0003134F')
}

func getFullUserNickname(user *telego.User) string {
	if user.LastName != "" {
		return fmt.Sprintf("%s %s", user.FirstName, user.LastName)
	}

	return user.FirstName
}
