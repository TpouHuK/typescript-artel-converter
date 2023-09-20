interface User<T extends B> {
  readonly name: string;
  foo1<B>(args: string): void;
  foo_nothing<B>(args: string);
  foo2: <X>(args?: string) => string | number;
}
