package emit

import jen "github.com/dave/jennifer/jen"

// Local aliases to avoid dot-importing jennifer while keeping concise calls.
type (
	Code  = jen.Code
	Dict  = jen.Dict
	Group = jen.Group
	File  = jen.File
)

var (
	NewFile = jen.NewFile
	Id      = jen.Id
	Lit     = jen.Lit
	Line    = jen.Line
	Func    = jen.Func
	For     = jen.For
	Range   = jen.Range
	Return  = jen.Return
	Nil     = jen.Nil
	String  = jen.String
	Int     = jen.Int
	Float64 = jen.Float64
	Bool    = jen.Bool
	Any     = jen.Any
	Map     = jen.Map
	Index   = jen.Index
	Qual    = jen.Qual
	Error   = jen.Error
	Case    = jen.Case
	Default = jen.Default
	Switch  = jen.Switch
	Var     = jen.Var
	If      = jen.If
	List    = jen.List
	Op      = jen.Op
	Comment = jen.Comment
)
