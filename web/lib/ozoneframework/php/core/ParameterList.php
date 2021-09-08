<?php

namespace Ozone\Framework;

/**
 * Parameters for the web request.
 *
 */
class ParameterList {

    private array $parameterArray = [];
    private array $parameterTypes = [];
    private array $parameterFrom = [];

    private array $allParameters = [];

    public function initParameterList($runData): void
    {
        if($runData->isAjaxMode()){
            $this->allParameters['AMODULE'] = [];
            foreach ($_POST as $key => $value) {
                $value = $this->fixNewLines($value);

                $this->parameterArray[$key] = $value;
                $this->parameterTypes[$key] = "AMODULE";
                $this->parameterFrom[$key] = 0; // 0 means "direct", + values means 'inherited'
                $this->allParameters['AMODULE'][$key] = $value;
            }
        } else{
            //initialize GET parameters from the url... because of mod_rewrite
            $qs =  $_SERVER['QUERY_STRING'];
            /* Check if there is a "?" char - if so, remove it. */
            $qs = preg_replace('/\?.*$/', '', $qs);
            $split = explode('/', $qs);
            if (count($split) >= 1) {
                $this->parameterArray['template'] = $split[0];
                $this->parameterTypes['template'] = 'GET';
            }

            /**
             * If an & is present in the URI, split that into key-value pairs.
             */
            $uri = $_SERVER['REQUEST_URI'];
            $uri = preg_replace('/^[^?]*\?/', '', $uri);
            $uriPairs = explode('&', $uri);
            foreach ($uriPairs as $uriPair) {
                $u = explode('=', $uriPair);
                $key = $u[0];
                $value = $u[1];
                $this->parameterArray[$key] = urldecode($value);
                $this->parameterTypes[$key] = 'GET';
                $this->parameterFrom[$key] = 0;
                $this->allParameters['GET'][$key] = urldecode($value);
            }

            // Parse path parameters.
            // This was edited to be more flexible in how it handles parameters, see
            // https://github.com/scpwiki/wikidot-path
            $this->allParameters['GET'] = [];
            for($i = 1; $i < count($split); $i += 2){
                $key = $split[$i];
                $value=$split[$i+1];
                $this->parameterArray[$key] = urldecode($value);
                $this->parameterTypes[$key] = 'GET';
                $this->parameterFrom[$key] = 0;
                $this->allParameters['GET'][$key] = urldecode($value);
            }

            // POST parameters are not affected by mod_rewrite
            $this->allParameters['POST'] = [];
            foreach ($_POST as $key => $value) {
                $value = $this->fixNewLines($value);
                $this->parameterArray[$key] = $value;
                $this->parameterTypes[$key] = 'POST';
                $this->parameterFrom[$key] = 0;
                $this->allParameters['POST'][$key] = urldecode($value);
            }
        }
    }

    /**
     * Converts an arbitrary parameter value to its appropriate PHP type.
     *
     * @param ?string $value The value to be converted
     * @return bool|int|string|null The value after conversion
     */
    private function convertValue(?string $value)
    {
        // Integer
        if (is_numeric($value)) {
            return intvalue($value);
        }

        // Boolean or Null
        switch ($value) {
            case 'yes':
            case 'true':
            case 't':
                return true;
            case 'no':
            case 'false':
            case 'f':
                return false;
            case null:
                return null;
        }

        // String
        return $value;
    }

    public function containsParameter(string $name): bool
    {
        return !array_search($name, $this->parameterArray) === false;
    }

    /**
     * @return mixed|null
     */
    public function getParameterValue(string $name, $type = null, $type2 = null)
    {
        if($type == null || $this->parameterTypes[$name] == $type || $this->parameterTypes[$name] == $type2){
            return $this->parameterArray[$name];
        }
        return null;
    }

    /**
     * Like getParameterValue(), but 'casts' the string to boolean.
     * Returns null if it doesn't exist or is another value..
     *
     * @param string $name
     * @return bool|null
     */
    public function getParameterValueBoolean(string $name): ?bool
    {
        switch ($this->getParameterValue($name)) {
            case 'true':
            case 't':
            case 'yes':
                return true;
            case 'false':
            case 'f':
            case 'no':
                return false;
            case null:
            default:
                return null;
        }
    }

    public function delParameter(string $key): void
    {
        unset($this->parameterArray[$key]);
        unset($this->parameterTypes[$key]);
    }

    /**
     * Returns type of the passed parameter: POST or GET.
     * @param $name
     * @return string
     */
    public function getParameterType(string $name): string
    {
        return $this->parameterTypes[$name];
    }

    public function asArray(): array
    {
        return $this->parameterArray;
    }

    public function asArrayAll(): array
    {
        return $this->allParameters;
    }

    public function addParameter(string $key, string $value, ?string $type=null): void
    {
        $this->parameterArray["$key"] = $value;
        $this->parameterTypes["$key"] = $type;
        $this->allParameters[$type][$key] = $value;
    }

    public function numberOfParameters(): int
    {
        return count($this->parameterArray);
    }

    private function fixNewLines(string $text): string
    {
        $text = str_replace("\r\n", "\n", $text);
        $text = str_replace("\r", "\n", $text);
        return $text;
    }

    public function getParametersByType(?string $type) {
        $out = [];
        foreach($this->parameterArray as $key => $value){
            if($this->parameterTypes[$key] === $type){
                $out[$key] = $value;
            }
        }
        return $out;
    }

    public function resolveParameter(string $key, string $from) {
        return $this->allParameters[$from][$key] ?? null;
    }
}
