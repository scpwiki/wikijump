<?php

declare(strict_types=1);

namespace Wikijump\Helpers;

use Ozone\Framework\PageProperties;
use Ozone\Framework\ParameterList;
use Ozone\Framework\PathManager;
use Wikidot\Utils\GlobalProperties;

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
        $this->initParameterList();

        /**
         * Then we take our extracted parameters and choose the template and screen to use.
         */
        $this->setTemplate();

        /** Set the language and theme for the request. */
        $this->language = $this->parameterArray['lang'] ?? GlobalProperties::$DEFAULT_LANGUAGE;
        if ($this->parameterArray['skin']) { $this->skin = $this->parameterArray['skin']; }

        $action = $this->parameterArray['action'];

        /** If there's a valid action... */
        if ($action !== null  && preg_match('/^[a-z0-9_\/]+$/i', $action) == 1) {
            unset($this->parameterArray['action']);
            unset($this->parameterTypes['action']);
            $this->action = str_replace("__", "/", $action);

             /** If the action is an Event (an AJAX call), assign the value to the property. */
            foreach ($this->parameterArray as $key => $value) {
                if ($key == 'event') {
                    $this->actionEvent = $value.'Event';
                }
            }
             /** Forms submissions come in with a different format, we'll unify them here. */
            foreach ($this->parameterArray as $key => $value) {
                if (str_starts_with('event_', $key)) {
                    $this->actionEvent = str_replace('event_', '', $key).'Event';
                }
            }
        }

        // initialize cookies...
        $this->cookies = $_COOKIE;

        // store original request uri and request method:
        $this->requestUri = $_SERVER['REQUEST_URI'];
        $this->requestMethod = $_SERVER['REQUEST_METHOD'];

    }

    /**
     * Retrieve contents of various PHP vars related to paths and queries.
     *
     * @return void
     */
    public function initParameterList(): void
    {
        /** The ajaxMode property can be set true by the AjaxModuleWebFlowController */
        if ($this->ajaxMode === true) {
            /** We populate an empty key for the parameter bucket that calling modules may use. */
            $this->allParameters['AMODULE'] = [];

            /** We break the POST down into key-value pairs and put them in our parameter store. */
            foreach ($_POST as $key => $value) {
                /** Convert any CRLF values to LF */
                $value = str_replace(["\r\n", "\r"], "\n", $value);

                /** Add this post key to the array. */
                $this->parameterArray[$key] = $value;

                /** And note that it's an AJAX Module. */
                $this->parameterTypes[$key] = 'AMODULE';

                /** Add the POST key-value pair as a subarray of an AMODULE key. */
                $this->allParameters['AMODULE'][$key] = $value;
            }
        } else {
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
                $value = str_replace(["\r\n", "\r", "\n"], "\n", $value);

                /** And add them to the bucket. */
                $this->parameterArray[$key] = $value;
                $this->parameterTypes[$key] = 'POST';
                $this->allParameters['POST'][$key] = urldecode($value);
            }
        }
    }

    /**
     *  Validate the requested template or screen and retrieve the location of it.
     */
    public function setTemplate()
    {
        /** The ajaxMode property can be set true by the AjaxModuleWebFlowController */
        if ($this->ajaxMode === true) {
            /** We can reliably assume that an AJAX module provided a key called moduleName */
            $template = $this->parameterArray['moduleName'];

            /**
             * If we didn't get a valid value back (not empty, contains only
             *  letters, numbers, underscores, and slashes), use the Empty template. (nothing)
             */
            if ($template == null || preg_match('/^[a-z0-9_\/]+$/i', $template) != 1) {
                $template = 'Empty';
            }

            /**
             * Otherwise we're going to assume there's a matching template in
             *  templates/modules to go with the PHP module in php/modules.
             *
             * These AJAX-centric modules generate the HTML to be injected back
             *  into the DOM wholesale.
             */
            $this->moduleTemplate = $template;

            $this->findModuleClass($this->moduleTemplate);
        } else {
            /**
             * If we're not making an AJAX call, we're looking for a Screen instead of a template.
             * Screens are in php/Screens and are the classes that use the
             * templates in templates/screens.
             */
            $template = $this->parameterArray['template'];

            /**
             * If we didn't get a valid value back (not empty, contains only
             *  letters, numbers, underscores, and slashes), use the Index screen. (nothing)
             */
            if ($template == null || preg_match('/^[a-z0-9_\/]+$/i', $template) != 1) {
                $template = 'Index';
            }

            /**
             * We receive the call on the php file as a path using two
             * underscores instead of a slash.  We need to pass the actual path to
             * our next set of tasks.
             */
            $template = str_replace('__', '/', $template);
            $this->screenTemplate = $template;

            $this->findModuleClass($this->screenTemplate);
        }
    }

    /**
     * Parse and return the corresponding path for a template or screen.
     * @param $template
     * @return void
     */
    public function findModuleClass($template)
    {
        /**  Retrieve the path to the module. */
        if ($this->moduleTemplate != null) {
            $classFilename = PathManager::moduleClass($template);
        }
        if ($this->screenTemplate != null) {
            $classFilename = PathManager::screenClass($this->screenTemplate);
        }

        /** If the file is there, return the path and name. */
        if (file_exists($classFilename)) {
            $moduleClassPath = $classFilename;
            $moduleClassName = array_slice(explode($classFilename, '/'), -1);
        } else {
            /** If it's not there, I guess we're going on an adventure. */
            if ($this->moduleTemplate != null) {
                $dir = PathManager::moduleClassDir();
            }
            if ($this->screenTemplate != null) {
                $dir = PathManager::screenClassDir();
            }

            /** We've got an arbitrary number of slashes that may lead to a file. */
            $module_path = explode('/', $template);

            /** We're going to work our way backwards from the deepest subfolder to the parent to find it. */
            for ($i = count($module_path) - 1; $i >= 0; $i--) {
                $path = '';
                /** For an incrementing value that's the same size as the max path depth... */
                for ($k = 0; $k < $i; $k++) {
                    /**
                     * Build a string with the most specific path to the
                     * fallback (DefaultModule.php) file.
                     */
                    $path .= $module_path[$k] . '/';
                    if ($this->moduleTemplate != null) {
                        $path .= 'DefaultModule.php';
                    }
                    if ($this->screenTemplate != null) {
                        $path .= 'DefaultScreen.php';
                    }
                }
                $classFiles[] = $path;
            }
            /** Add this constructed path to an array. */

            foreach ($classFiles as $classFile) {
                /** The first one that matches is the one we'll provide to the caller. */
                if (file_exists($dir . $classFile)) {
                    $moduleClassPath = $dir . $classFile;
                    if($this->moduleTemplate != null)
                    {
                        $moduleClassName = 'DefaultModule';
                    }
                    if($this->screenTemplate != null)
                    {
                        $moduleClassName = 'DefaultScreen';
                    }
                    break;
                }
            }
            $this->moduleClassName = $moduleClassName;
            $this->moduleClassPath = $moduleClassPath;
        }
    }
}
