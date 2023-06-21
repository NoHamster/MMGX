
@MGX{
	EQ<A,A> true
	@impl EQ<0..100>

	FOREACH<I>(M, A, ...) M(A) FOREACH<I-1>(M, ...)

	Add<a,b> @eval(a+b)

	@impl FOREACH<1..10>

	@export EQUAL(A,B) EQ<A,B>
	MGXFUNC(...) ...
}

