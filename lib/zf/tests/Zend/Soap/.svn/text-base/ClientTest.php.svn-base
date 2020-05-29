<?php
/**
 * Zend Framework
 *
 * LICENSE
 *
 * This source file is subject to the new BSD license that is bundled
 * with this package in the file LICENSE.txt.
 * It is also available through the world-wide-web at this URL:
 * http://framework.zend.com/license/new-bsd
 * If you did not receive a copy of the license and are unable to
 * obtain it through the world-wide-web, please send an email
 * to license@zend.com so we can send you a copy immediately.
 *
 * @category   Zend
 * @package    UnitTests
 * @copyright  Copyright (c) 2005-2008 Zend Technologies USA Inc. (http://www.zend.com)
 * @license    http://framework.zend.com/license/new-bsd     New BSD License
 */

require_once dirname(__FILE__)."/../../TestHelper.php";

/** PHPUnit Test Case */
require_once "PHPUnit/Framework/TestCase.php";

/** Zend_Soap_Server */
require_once 'Zend/Soap/Server.php';

/** Zend_Soap_Client */
require_once 'Zend/Soap/Client.php';

/**
 * Zend_Soap_Client
 *
 * @category   Zend
 * @package    UnitTests
 * @version    $Id$
 */
class Zend_Soap_ClientTest extends PHPUnit_Framework_TestCase
{
    public function setUp()
    {
        if (!extension_loaded('soap')) {
           $this->markTestSkipped('SOAP Extension is not loaded');
        }
    }

    public function testSetOptions()
    {
    	/*************************************************************
    	 * ------ Test WSDL mode options -----------------------------
    	 *************************************************************/
    	$client = new Zend_Soap_Client();

    	$this->assertTrue($client->getOptions() == array('encoding' => 'UTF-8', 'soap_version' => SOAP_1_2));

    	$nonWsdlOptions = array('soap_version'   => SOAP_1_1,
		                        'classmap'       => array('TestData1' => 'Zend_Soap_Client_TestData1',
		                                            'TestData2' => 'Zend_Soap_Client_TestData2',),
		                        'encoding'       => 'ISO-8859-1',
		                        'uri'            => 'http://framework.zend.com/Zend_Soap_ServerTest.php',
		                        'location'       => 'http://framework.zend.com/Zend_Soap_ServerTest.php',
		                        'use'            => SOAP_ENCODED,
		                        'style'          => SOAP_RPC,

		                        'login'          => 'http_login',
		                        'password'       => 'http_password',

		                        'proxy_host'     => 'proxy.somehost.com',
    	                        'proxy_port'     => 8080,
    	                        'proxy_login'    => 'proxy_login',
    	                        'proxy_password' => 'proxy_password',

		                        'local_cert'     => dirname(__FILE__).'/_files/cert_file',
		                        'passphrase'     => 'some pass phrase',

		                        'compression'    => SOAP_COMPRESSION_ACCEPT | SOAP_COMPRESSION_GZIP | 5);

    	$client->setOptions($nonWsdlOptions);
    	$this->assertTrue($client->getOptions() == $nonWsdlOptions);


        /*************************************************************
         * ------ Test non-WSDL mode options -----------------------------
         *************************************************************/
        $client1 = new Zend_Soap_Client();

        $this->assertTrue($client1->getOptions() == array('encoding' => 'UTF-8', 'soap_version' => SOAP_1_2));

        $wsdlOptions = array('soap_version'   => SOAP_1_1,
                             'wsdl'           => dirname(__FILE__).'/_files/wsdl_example.wsdl',
                             'classmap'       => array('TestData1' => 'Zend_Soap_Client_TestData1',
                                                 'TestData2' => 'Zend_Soap_Client_TestData2',),
                             'encoding'       => 'ISO-8859-1',

                             'login'          => 'http_login',
                             'password'       => 'http_password',

                             'proxy_host'     => 'proxy.somehost.com',
                             'proxy_port'     => 8080,
                             'proxy_login'    => 'proxy_login',
                             'proxy_password' => 'proxy_password',

                             'local_cert'     => dirname(__FILE__).'/_files/cert_file',
                             'passphrase'     => 'some pass phrase',

                             'compression'    => SOAP_COMPRESSION_ACCEPT | SOAP_COMPRESSION_GZIP | 5);

        $client1->setOptions($wsdlOptions);
        $this->assertTrue($client1->getOptions() == $wsdlOptions);
    }

    public function testGetOptions()
    {
    	$client = new Zend_Soap_Client();

        $this->assertTrue($client->getOptions() == array('encoding' => 'UTF-8', 'soap_version' => SOAP_1_2));

        $options = array('soap_version'   => SOAP_1_1,
                         'wsdl'           => dirname(__FILE__).'/_files/wsdl_example.wsdl',

                         'classmap'       => array('TestData1' => 'Zend_Soap_Client_TestData1',
                                             'TestData2' => 'Zend_Soap_Client_TestData2',),
                         'encoding'       => 'ISO-8859-1',
                         'uri'            => 'http://framework.zend.com/Zend_Soap_ServerTest.php',
                         'location'       => 'http://framework.zend.com/Zend_Soap_ServerTest.php',
                         'use'            => SOAP_ENCODED,
                         'style'          => SOAP_RPC,

                         'login'          => 'http_login',
                         'password'       => 'http_password',

                         'proxy_host'     => 'proxy.somehost.com',
                         'proxy_port'     => 8080,
                         'proxy_login'    => 'proxy_login',
                         'proxy_password' => 'proxy_password',

                         'local_cert'     => dirname(__FILE__).'/_files/cert_file',
                         'passphrase'     => 'some pass phrase',

                         'compression'    => SOAP_COMPRESSION_ACCEPT | SOAP_COMPRESSION_GZIP | 5);

        $client->setOptions($options);
        $this->assertTrue($client->getOptions() == $options);
    }

    public function testGetFunctions()
    {
        $server = new Zend_Soap_Server(dirname(__FILE__) . '/_files/wsdl_example.wsdl');
        $server->setClass('Zend_Soap_Client_TestClass');

        $client = new Zend_Soap_Client_Local($server, dirname(__FILE__) . '/_files/wsdl_example.wsdl');

        $this->assertTrue($client->getFunctions() == array('string testFunc1()',
                                                           'string testFunc2(string $who)',
                                                           'string testFunc3(string $who, int $when)',
                                                           'string testFunc4()'));
    }

    /**
     * @todo Implement testGetTypes().
     */
    public function testGetTypes()
    {
    	// Remove the following line when you implement this test.
        $this->markTestIncomplete(
          "This test has not been implemented yet."
        );
    }

    public function testGetLastRequest()
    {
    	$server = new Zend_Soap_Server(dirname(__FILE__) . '/_files/wsdl_example.wsdl');
        $server->setClass('Zend_Soap_Client_TestClass');

        $client = new Zend_Soap_Client_Local($server, dirname(__FILE__) . '/_files/wsdl_example.wsdl');

        // Perform request
        $client->testFunc2('World');

        $expectedRequest = '<?xml version="1.0" encoding="UTF-8"?>' . "\n"
                         . '<env:Envelope xmlns:env="http://www.w3.org/2003/05/soap-envelope" '
                         .               'xmlns:xsd="http://www.w3.org/2001/XMLSchema" '
                         .               'xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" '
                         .               'xmlns:enc="http://www.w3.org/2003/05/soap-encoding">'
                         .     '<env:Body>'
                         .         '<env:testFunc2 env:encodingStyle="http://www.w3.org/2003/05/soap-encoding">'
                         .             '<who xsi:type="xsd:string">World</who>'
                         .         '</env:testFunc2>'
                         .     '</env:Body>'
                         . '</env:Envelope>' . "\n";

        $this->assertEquals($client->getLastRequest(), $expectedRequest);
    }

    public function testGetLastResponse()
    {
    	$server = new Zend_Soap_Server(dirname(__FILE__) . '/_files/wsdl_example.wsdl');
        $server->setClass('Zend_Soap_Client_TestClass');

        $client = new Zend_Soap_Client_Local($server, dirname(__FILE__) . '/_files/wsdl_example.wsdl');

        // Perform request
        $client->testFunc2('World');

        $expectedResponse = '<?xml version="1.0" encoding="UTF-8"?>' . "\n"
                          . '<env:Envelope xmlns:env="http://www.w3.org/2003/05/soap-envelope" '
                          .               'xmlns:xsd="http://www.w3.org/2001/XMLSchema" '
                          .               'xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" '
                          .               'xmlns:enc="http://www.w3.org/2003/05/soap-encoding">'
                          .     '<env:Body xmlns:rpc="http://www.w3.org/2003/05/soap-rpc">'
                          .         '<env:testFunc2Response env:encodingStyle="http://www.w3.org/2003/05/soap-encoding">'
                          .             '<rpc:result>testFunc2Return</rpc:result>'
                          .             '<testFunc2Return xsi:type="xsd:string">Hello World!</testFunc2Return>'
                          .         '</env:testFunc2Response>'
                          .     '</env:Body>'
                          . '</env:Envelope>' . "\n";

        $this->assertEquals($client->getLastResponse(), $expectedResponse);
    }

    public function testCallInvoke()
    {
    	$server = new Zend_Soap_Server(dirname(__FILE__) . '/_files/wsdl_example.wsdl');
        $server->setClass('Zend_Soap_Client_TestClass');

        $client = new Zend_Soap_Client_Local($server, dirname(__FILE__) . '/_files/wsdl_example.wsdl');

        $this->assertEquals($client->testFunc2('World'), 'Hello World!');
    }
}


/** Test Class */
class Zend_Soap_Client_TestClass {
    /**
     * Test Function 1
     *
     * @return string
     */
    function testFunc1()
    {
        return "Hello World";
    }

    /**
     * Test Function 2
     *
     * @param string $who Some Arg
     * @return string
     */
    function testFunc2($who)
    {
        return "Hello $who!";
    }

    /**
     * Test Function 3
     *
     * @param string $who Some Arg
     * @param int $when Some
     * @return string
     */
    function testFunc3($who, $when)
    {
        return "Hello $who, How are you $when";
    }

    /**
     * Test Function 4
     *
     * @return string
     */
    static function testFunc4()
    {
        return "I'm Static!";
    }
}

/** Test class 2 */
class Zend_Soap_Client_TestData1 {
    /**
     * Property1
     *
     * @var string
     */
     public $property1;

    /**
     * Property2
     *
     * @var float
     */
     public $property2;
}

/** Test class 2 */
class Zend_Soap_Client_TestData2 {
    /**
     * Property1
     *
     * @var integer
     */
     public $property1;

    /**
     * Property1
     *
     * @var float
     */
     public $property2;
}


/* Test Functions */

/**
 * Test Function
 *
 * @param string $arg
 * @return string
 */
function Zend_Soap_Client_TestFunc1($who)
{
    return "Hello $who";
}

/**
 * Test Function 2
 */
function Zend_Soap_Client_TestFunc2()
{
    return "Hello World";
}

/**
 * Return false
 *
 * @return bool
 */
function Zend_Soap_Client_TestFunc3()
{
    return false;
}

/**
 * Return true
 *
 * @return bool
 */
function Zend_Soap_Client_TestFunc4()
{
    return true;
}

/**
 * Return integer
 *
 * @return int
 */
function Zend_Soap_Client_TestFunc5()
{
    return 123;
}

/**
 * Return string
 *
 * @return string
 */
function Zend_Soap_Client_TestFunc6()
{
    return "string";
}

