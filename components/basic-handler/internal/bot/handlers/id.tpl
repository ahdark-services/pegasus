{{ if .Message -}}
当前信息 ID：{{ .Message.MessageID }}
聊天 ID：{{ .Message.Chat.ID }}
聊天类型：{{ .Message.Chat.Type }}
聊天 Title：{{ .Message.Chat.Title }}
{{ if .Message.From }}
发送者 ID：{{ .Message.From.ID }}
用户：{{ .Message.From.FirstName }}{{ with .Message.From.LastName }} {{ . }}{{ end }}
用户名：@{{ .Message.From.Username }}
语言：{{ .Message.From.LanguageCode }}
是否 Premium 用户：{{ if .Message.From.IsPremium }}是{{ else }}否{{ end }}
{{- end }}

{{ if .Message.ReplyToMessage -}}
回复的信息 ID：{{ .Message.ReplyToMessage.MessageID }}
{{ if .Message.ReplyToMessage.From -}}
回复的信息发送者 ID：{{ .Message.ReplyToMessage.From.ID }}
回复的信息发送者用户：{{ .Message.ReplyToMessage.From.FirstName }}{{ with .Message.ReplyToMessage.From.LastName }} {{ . }}{{ end }}
回复的信息发送者用户名：@{{ .Message.ReplyToMessage.From.Username }}
{{- end }}
{{ if .Message.ReplyToMessage.SenderChat -}}
回复的信息发送者聊天 ID：{{ .Message.ReplyToMessage.SenderChat.ID }}
回复的信息发送者聊天类型：{{ .Message.ReplyToMessage.SenderChat.Type }}
回复的信息发送者聊天 Title：{{ .Message.ReplyToMessage.SenderChat.Title }}
{{- end }}
{{- end }}
{{- end }}
