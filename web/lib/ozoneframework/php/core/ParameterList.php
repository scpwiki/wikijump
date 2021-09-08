<?php

namespace Ozone\Framework;





/**
 * Parameters for the web request.
 *
 */
class ParameterList {

	private $parameterArray = [];
	private $parameterTypes = [];
	private $parameterFrom = [];

	private $allParameters = [];

	public function initParameterList($runData) {

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
			$splited = explode('/', $qs);
			if(count($splited)>= 1){
				$this->parameterArray['template'] = $splited[0];
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
			// now populate other parameters...
			$this->allParameters['GET'] = [];
			for($i=1; $i<count($splited); $i+=2){
				$key = $splited[$i];
				$value=$splited[$i+1];
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

	public function containsParameter($name): bool
    {
		return !array_search($name, $this->parameterArray) === false;
	}

	public function getParameterValue(string $name, $type = null, $type2 = null) {
		if($type == null || $this->parameterTypes[$name] == $type || $this->parameterTypes[$name] == $type2){
			return $this->parameterArray[$name];
		}
		return null;
	}

    public function getParameterValueBoolean(string $name): ?bool
    {
        switch ($this->getParameterValue($name)) {
            case 'true':
                return true;
            case 'false':
                return false;
            case null:
            default:
                return null;
        }
    }

	public function delParameter($key){
		unset($this->parameterArray[$key]);
		unset($this->parameterTypes[$key]);
	}

    /**
     * Returns type of the passed parameter: POST or GET.
     * @param $name
     * @return string
     */
	public function getParameterType($name) : string {
		return $this->parameterTypes[$name];
	}

	public function asArray() {
		return $this->parameterArray;
	}

	public function asArrayAll(){
		return $this->allParameters;
	}

	public function addParameter($key, $value, $type=null){
			$this->parameterArray["$key"] = $value;
			$this->parameterTypes["$key"] = $type;
			$this->allParameters[$type][$key] = $value;
	}

	public function numberOfParameters(){
		return count($this->parameterArray);
	}

	private function fixNewLines($text){
		$text = str_replace("\r\n", "\n", $text);
		$text = str_replace("\r", "\n", $text);
		return $text;
	}

	public function getParametersByType($type){
		$out = [];
		foreach($this->parameterArray as $key => $value){
			if($this->parameterTypes[$key] === $type){
				$out[$key] = $value;
			}
		}
		return $out;
	}

	public function resolveParameter($key, $from) {
		if(isset($this->allParameters[$from][$key])) {
			return $this->allParameters[$from][$key];
		} else {
 			return null;
		}
	}

}
