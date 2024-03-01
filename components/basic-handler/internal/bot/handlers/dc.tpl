{{ if ne .Message.Chat.Username "" -}}
此群组所在数据中心为 DC{{ datacenter .Message.Chat.Username }}
{{ else -}}
此群组未设置用户名
{{- end }}

{{ if and .Message.From (ne .Message.From.Username "") -}}
您所在数据中心为 DC{{ datacenter .Message.From.Username }}
{{ else -}}
您未设置用户名
{{- end }}

此数据中心数据通过聊天头像查询，不保证准确性。