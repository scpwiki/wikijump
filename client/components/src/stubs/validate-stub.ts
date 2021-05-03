import * as YAML from "yaml"
import { validate } from "jsonschema"

import schemaFile from "./stubs.schema.yaml"
const schema = YAML.parse(schemaFile)

/**
 * Checks whether a HTML element conforms to a given specification.
 *
 * @param element - The element to check.
 * @param stubSpec - The specification.
 * @see stubs.schema.yaml
 */
export function validateStub(element: HTMLElement, stubSpec: string) {
  // TODO This function must be able to accept an array of elements
  //  (because of repeats)
  const spec = YAML.parse(stubSpec)
  // Check that the stub spec matches the schema
  if (!validateStubAgainstSchema(spec, schema)) {
    throw new Error("The spec does not match the schema")
  }
  // We know the spec matches the schema, so in lieu of generating TypeScript
  // types for it, we can be confident in asserting that it has certain
  // properties
  return validateElement(element, spec.root)
}

/**
 * Checks that a given specification is compliant with the specification
 * schema.
 *
 * @param spec - The parsed specification.
 * @param schema - The parsed schema.
 */
function validateStubAgainstSchema(spec: any, schema: any) {
  const { errors, valid } = validate(spec, schema)
  // If there are errors, print them for inspection
  if (!valid) {
    console.log(errors)
  }
  return valid
}

/**
 * Checks that a HTML element conforms to a specification. Assumes that the
 * specification is compliant with the schema.
 *
 * @param element - The element to be checked.
 * @param spec - The specification to check the element against.
 */
function validateElement(element: HTMLElement, spec: any) {
  // Check that the element has a matching tag
  if ("tag" in spec) {
    if (element.tagName !== spec.tag.toUpperCase()) {
      throw new Error("Element tag does not match")
    }
  }
  // Check that the element has matching attributes
  if ("attributes" in spec) {
    validateAttributes(element.attributes, spec.attributes)
  }
  // Check that the children of the element are valid
  if ("children" in spec) {
    if (spec.children === true) {
      // Arbitrary children are allowed; no validation to perform
    } else if (spec.children === false) {
      // No children are permitted
      if (element.children.length > 0) {
        throw new Error("Element has children but none are permitted")
      }
    } else if (Array.isArray(spec.children)) {
      // The spec is an array representing a series of child definitions,
      // each of which must be present
      // TODO
    } else {
      // The spec is an object representing a single child definition
      // TODO
      // Need to check not that this element contains a single child, but
      // that it contains a single type of child that is consistent with
      // this definition.
      // Would probably be best to pass an array of elements to either
      // validateStub, validateElement, or a new validateElements
    }
  } else if (element.children.length > 0) {
    // No children are permitted
    throw new Error("Element has children but none are permitted")
  }
}

/**
 * Checks that a set of HTML attributes are compliant with a specification.
 *
 * @param attributes - The attributes to check.
 * @param attributesSpec - The attributes specification, which is assumed
 * to be compliant with the schema.
 * @param prefix - A string to prepend to the name of each attribute in the
 * spec when comparing it with the provided attributes.
 * @returns true, or throws an error if the attributes do not match.
 */
function validateAttributes(attributes: NamedNodeMap, attributesSpec: any, prefix = "") {
  Object.entries(attributesSpec).forEach(([key, value]) => {
    if (key === "data") {
      // The 'data' namespace has nested values
      validateAttributes(attributes, attributesSpec.data, "data-")
    }
    key = prefix + key
    if (Boolean(value)) {
      if (typeof value === "string") {
        // The attribute value must match this string
        if (attributes.getNamedItem(key)?.value !== value) {
          throw new Error(`Attribute ${key} expected to be ${value}`)
        }
      }
      // If the value is truthy but is not a string, it is true
      // The attribute value can be anything, but must be present
      if (attributes.getNamedItem(key) === null) {
        throw new Error(`Attribute ${key} expected`)
      }
    }
  })
  return true
}
