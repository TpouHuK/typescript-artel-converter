interface User<T> {
  readonly name: string;
  foo1<B>(args: string): void;
  foo2: <X>(args: string) => string;
}
