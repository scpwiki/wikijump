<?php

namespace Wikijump\Helpers;

use Ozone\Framework\PageProperties;
use Ozone\Framework\ParameterList;

/**
 * PageBuilder: It's like RunData, but good.
 * @package Wikijump\Helpers
 */
class PageBuilder
{
    /**
     * PageBuilder properties:
     */
    private $parameterList;
    private $screenTemplate;
    private $screenClassName;
    private $screenClassPath;
    private $moduleTemplate;
    private $moduleClassName;
    private $moduleClassPath;
    private $context;
    private $action;
    private $actionEvent;
    private $nextAction;
    private $nextActionEvent;
    private $errorMessages = [];
    private $messages = [];
    private $page;
    private $cookies;
    private $language;
    private $session = null;
    private bool $ajaxMode = false;
    private $ajaxResponse;
    private $requestUri;
    private $requestMethod;
    private $extra = [];
    private bool $formToolHttpProcessed = false;
    private $temp; // temporary variables
    private $_outCookies = [];

    /**
     * PageProperties properties.
     * @see PageProperties
     */
    private string $title = 'No title';
    private array $styles1 = [];
    private array $styles2 = [];
    private array $jsFiles = [];
    private array $meta = [];
    private array $httpEquivs = [];
    private array $bodyProperties = [];
    private array $links = [];
    private string $skin = 'default';
    private string $layout = 'Default';
    private array $styleRaw1 = [];
    private array $styleRaw2 = [];
    private array $jsRaw = [];
    private array $headRaw = [];

    /**
     * ParameterList properties
     * @see ParameterList
     */
    private array $parameterArray = [];
    private array $parameterTypes = [];
    private array $allParameters = [];

    public function init()
    {
        /**
         * First, the parameter list is populated.
         * This is the business of inspecting the request and making some decisions.
         */
        $this->parameterList = $this->initParameterList();
    }

    private function initParameterList()
    {
        /** The ajaxMode property can be set true by the AjaxModuleWebFlowController */
        if($this->ajaxMode === true){
            $this->allParameters['AMODULE'] = [];
            foreach ($_POST as $key => $value) {
                /** Convert any CRLF values to LF */
                $value = str_replace(["\r\n", "\r"], "\n", $value);

                /** Add this post key to the array. */
                $this->parameterArray[$key] = $value;

                /** And note that it's an AJAX Module. */
                $this->parameterTypes[$key] = "AMODULE";

                /** Add the POST key-value pair as a subarray of an AMODULE key. */
                $this->allParameters['AMODULE'][$key] = $value;
            }
        }
        else
        {
            /** We'll instantiate some empty keys that some calling modules expect. */
            $this->allParameters['GET'] = [];
            $this->allParameters['POST'] = [];

            /**
             * Nginx does a transform on the request for us.
             *
             * The user makes a request for a page like:
             * `wikijump.test/testpage`
             *
             * But nginx transforms it and provides it to the server as:
             * `wikijump.test/index.php?Wiki__WikiScreen/wiki_page/testpage`
             *
             * And so the query string looks like:
             *          0           1         2
             * `Wiki__WikiScreen/wiki_page/testpage`
             *
             * Here we break the request into its component parts.
             */
            $page_parameters = explode('/', $_SERVER['QUERY_STRING']);
            $screen_type = $page_parameters[0];
            $asset_type = $page_parameters[1];
            $asset_name = $page_parameters[2];

            /** And we populate the parameter bucket with the results. */
            $this->parameterArray['template'] = $screen_type;
            $this->parameterTypes['template'] = 'GET';
            $this->parameterArray[$asset_type] = urldecode($asset_name);
            $this->parameterTypes[$asset_type] = 'GET';
            $this->allParameters['GET'][$asset_type] = urldecode($asset_name);

            /**
             * For a "real" GET request (one where the user provides a GET string),
             * The value of it is available in `$_SERVER['REQUEST_URI'].
             * However it also includes the reqested page name, which we don't need right now.
             */
            $get_string = explode('?', $_SERVER['REQUEST_URI'])[1];

            /** We'll break the GET string into key-value pairs. */
            $get_pairs = explode('&', $get_string);

            /** Then break the key-value pairs into their respective items. */
            foreach ($get_pairs as $get_pair) {
                $item = explode('=', $get_pair);
                $key = $item[0];
                $value = $item[1];

                /** We put the set in the same parameter store we're already building. */
                $this->parameterArray[$key] = urldecode($value);
                $this->parameterTypes[$key] = 'GET';
                $this->allParameters['GET'][$key] = urldecode($value);
            }

            /** Lastly, POST values don't need as much manipulation. */
            foreach ($_POST as $key => $value) {

                /** Convert any CRLF values to LF */
                $value = str_replace(["\r\n", "\r"], "\n", $value);

                /** And add them to the bucket. */
                $this->parameterArray[$key] = $value;
                $this->parameterTypes[$key] = 'POST';
                $this->allParameters['POST'][$key] = urldecode($value);
            }
        }
    }
}
