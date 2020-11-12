import OZONE from "@/javascript/OZONE";

describe("arrayToPostData", () => {
  it("converts an object to a query string", () => {
    const obj = { one: "two", three: "four" };
    expect(OZONE.utils.arrayToPostData(obj)).toEqual("one=two&three=four");
  });
});
