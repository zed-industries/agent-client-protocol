package util

import (
	"strings"
	"unicode"
)

// SanitizeComment removes backticks and normalizes whitespace for Go comments.
func SanitizeComment(s string) string {
	s = strings.ReplaceAll(s, "`", "'")
	lines := strings.Split(s, "\n")
	for i := range lines {
		lines[i] = strings.TrimSpace(lines[i])
	}
	return strings.Join(lines, " ")
}

// TitleWord uppercases the first rune and lowercases the rest.
func TitleWord(s string) string {
	if s == "" {
		return s
	}
	r := []rune(s)
	r[0] = unicode.ToUpper(r[0])
	for i := 1; i < len(r); i++ {
		r[i] = unicode.ToLower(r[i])
	}
	return string(r)
}

// SplitCamel splits a camelCase string into tokens.
func SplitCamel(s string) []string {
	var parts []string
	last := 0
	for i := 1; i < len(s); i++ {
		if isBoundary(s[i-1], s[i]) {
			parts = append(parts, s[last:i])
			last = i
		}
	}
	parts = append(parts, s[last:])
	return parts
}

func isBoundary(prev, curr byte) bool {
	return (prev >= 'a' && prev <= 'z' && curr >= 'A' && curr <= 'Z') || curr == '_'
}

// ToExportedField converts snake_case or camelCase to PascalCase.
func ToExportedField(name string) string {
	parts := strings.Split(name, "_")
	if len(parts) == 1 {
		parts = SplitCamel(name)
	}
	for i := range parts {
		parts[i] = TitleWord(parts[i])
	}
	return strings.Join(parts, "")
}

// ToEnumConst builds a const identifier like <TypeName><Value>.
func ToEnumConst(typeName, val string) string {
	cleaned := make([]rune, 0, len(val))
	for _, r := range val {
		if (r >= 'a' && r <= 'z') || (r >= 'A' && r <= 'Z') || (r >= '0' && r <= '9') {
			cleaned = append(cleaned, r)
		} else {
			cleaned = append(cleaned, '_')
		}
	}
	parts := strings.FieldsFunc(string(cleaned), func(r rune) bool { return r == '_' })
	for i := range parts {
		parts[i] = TitleWord(strings.ToLower(parts[i]))
	}
	return typeName + strings.Join(parts, "")
}
