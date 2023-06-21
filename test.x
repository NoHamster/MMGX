#include <stdio.h>


@MGX_T {
	//F(a)
	Function(a,b,c, ...) F(a) b c ...
	Func<A,B,A> F<A> B

	@impl Func<0..10, 2.. 10>
	@export Func2(x) x

	@use MGX

	FUNC(A,B) MGX.MGXFUNC(A,A,A,B,B,B)

	FUNC2(A,B) MGX::MGXFUNC(A,A,A,B,B,B)
}
