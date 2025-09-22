package emit

import (
	"github.com/zed-industries/agent-client-protocol/go/cmd/generate/internal/ir"
)

// invInvalid: return invalid params with compact json-like message
func jInvInvalid() Code {
	return Return(Nil(), Id("NewInvalidParams").Call(Map(String()).Any().Values(Dict{Lit("error"): Id("err").Dot("Error").Call()})))
}

// retToReqErr: wrap error to JSON-RPC request error
func jRetToReqErr() Code { return Return(Nil(), Id("toReqErr").Call(Id("err"))) }

// jUnmarshalValidate emits var p T; json.Unmarshal; p.Validate
func jUnmarshalValidate(typeName string) []Code {
	return []Code{
		Var().Id("p").Id(typeName),
		If(List(Id("err")).Op(":=").Qual("encoding/json", "Unmarshal").Call(Id("params"), Op("&").Id("p")), Id("err").Op("!=").Nil()).
			Block(jInvInvalid()),
		If(List(Id("err")).Op(":=").Id("p").Dot("Validate").Call(), Id("err").Op("!=").Nil()).
			Block(jInvInvalid()),
	}
}

// jAgentAssert returns prelude for interface assertions and the receiver name.
func jAgentAssert(binding ir.MethodBinding) ([]Code, string) {
	switch binding {
	case ir.BindAgentLoader:
		return []Code{
			List(Id("loader"), Id("ok")).Op(":=").Id("a").Dot("agent").Assert(Id("AgentLoader")),
			If(Op("!").Id("ok")).Block(Return(Nil(), Id("NewMethodNotFound").Call(Id("method")))),
		}, "loader"
	case ir.BindAgentExperimental:
		return []Code{
			List(Id("exp"), Id("ok")).Op(":=").Id("a").Dot("agent").Assert(Id("AgentExperimental")),
			If(Op("!").Id("ok")).Block(Return(Nil(), Id("NewMethodNotFound").Call(Id("method")))),
		}, "exp"
	default:
		return nil, "a.agent"
	}
}

// jClientAssert returns prelude for interface assertions and the receiver name.
func jClientAssert(binding ir.MethodBinding) ([]Code, string) {
	switch binding {
	case ir.BindClientExperimental:
		return []Code{
			List(Id("exp"), Id("ok")).Op(":=").Id("c").Dot("client").Assert(Id("ClientExperimental")),
			If(Op("!").Id("ok")).Block(Return(Nil(), Id("NewMethodNotFound").Call(Id("method")))),
		}, "exp"
	case ir.BindClientTerminal:
		return []Code{
			List(Id("t"), Id("ok")).Op(":=").Id("c").Dot("client").Assert(Id("ClientTerminal")),
			If(Op("!").Id("ok")).Block(Return(Nil(), Id("NewMethodNotFound").Call(Id("method")))),
		}, "t"
	default:
		return nil, "c.client"
	}
}

// Request call emitters for handlers
func jCallRequestNoResp(recv, methodName string) []Code {
	return []Code{
		If(List(Id("err")).Op(":=").Id(recv).Dot(methodName).Call(Id("ctx"), Id("p")), Id("err").Op("!=").Nil()).Block(jRetToReqErr()),
		Return(Nil(), Nil()),
	}
}

func jCallRequestWithResp(recv, methodName string) []Code {
	return []Code{
		List(Id("resp"), Id("err")).Op(":=").Id(recv).Dot(methodName).Call(Id("ctx"), Id("p")),
		If(Id("err").Op("!=").Nil()).Block(jRetToReqErr()),
		Return(Id("resp"), Nil()),
	}
}

func jCallNotification(recv, methodName string) []Code {
	return []Code{
		If(List(Id("err")).Op(":=").Id(recv).Dot(methodName).Call(Id("ctx"), Id("p")), Id("err").Op("!=").Nil()).Block(jRetToReqErr()),
		Return(Nil(), Nil()),
	}
}
