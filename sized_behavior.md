# Definitions

customsec ::= section_0(custom)
custom ::= name byte\*

codesec ::= section_10(vec(code))
code ::= size:u32 func (if size = ||func||)
func ::= vec(locals) expr
locals ::= n:u32 valtype

# Tests

Code missing end marker
