<?php
// Call Zend_Filter_Word_UnderscoreToPathSeparatorTest::main() if this source file is executed directly.
if (!defined("PHPUnit_MAIN_METHOD")) {
    require_once dirname(dirname(dirname(__FILE__))) . '/TestHelper.php';
    define("PHPUnit_MAIN_METHOD", "Zend_Filter_Word_UnderscoreToPathSeparatorTest::main");
}

require_once "PHPUnit/Framework/TestCase.php";
require_once "PHPUnit/Framework/TestSuite.php";

require_once 'Zend/Filter/Word/UnderscoreToPathSeparator.php';

/**
 * Test class for Zend_Filter_Word_UnderscoreToPathSeparator.
 */
class Zend_Filter_Word_UnderscoreToPathSeparatorTest extends PHPUnit_Framework_TestCase 
{
    /**
     * Runs the test methods of this class.
     *
     * @access public
     * @static
     */
    public static function main() 
    {
        require_once "PHPUnit/TextUI/TestRunner.php";

        $suite  = new PHPUnit_Framework_TestSuite("Zend_Filter_Word_UnderscoreToPathSeparatorTest");
        $result = PHPUnit_TextUI_TestRunner::run($suite);
    }

    public function testFilterSeparatesCamelCasedWordsWithPathSeparators()
    {
        $string   = 'underscore_separated_words';
        $filter   = new Zend_Filter_Word_UnderscoreToPathSeparator();
        $filtered = $filter->filter($string);

        $this->assertNotEquals($string, $filtered);
        $expected = 'underscore' 
                  . DIRECTORY_SEPARATOR . 'separated'
                  . DIRECTORY_SEPARATOR . 'words';
        $this->assertEquals($expected, $filtered);
    }
}

// Call Zend_Filter_Word_UnderscoreToPathSeparatorTest::main() if this source file is executed directly.
if (PHPUnit_MAIN_METHOD == "Zend_Filter_Word_UnderscoreToPathSeparatorTest::main") {
    Zend_Filter_Word_UnderscoreToPathSeparatorTest::main();
}
