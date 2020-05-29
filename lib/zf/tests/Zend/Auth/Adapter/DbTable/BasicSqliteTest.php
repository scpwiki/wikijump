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
 * @package    Zend_Auth
 * @subpackage UnitTests
 * @copyright  Copyright (c) 2005-2008 Zend Technologies USA Inc. (http://www.zend.com)
 * @license    http://framework.zend.com/license/new-bsd     New BSD License
 * @version    $Id: BasicSqliteTest.php 8064 2008-02-16 10:58:39Z thomas $
 */


/**
 * PHPUnit_Framework_TestCase
 */
require_once 'PHPUnit/Framework/TestCase.php';


/**
 * @see Zend_Db_Adapter_Pdo_Sqlite
 */
require_once 'Zend/Db/Adapter/Pdo/Sqlite.php';


/**
 * This is required because Zend_Db_Adapter_Pdo_Sqlite uses Zend_Db constants
 * but does not load the file containing the Zend_Db class.
 *
 * @see Zend_Db
 */
require_once 'Zend/Db.php';


/**
 * @see Zend_Auth_Adapter_DbTable
 */
require_once 'Zend/Auth/Adapter/DbTable.php';


/**
 * @category   Zend
 * @package    Zend_Auth
 * @subpackage UnitTests
 * @copyright  Copyright (c) 2005-2008 Zend Technologies USA Inc. (http://www.zend.com)
 * @license    http://framework.zend.com/license/new-bsd     New BSD License
 */
class Zend_Auth_Adapter_DbTable_BasicSqliteTest extends PHPUnit_Framework_TestCase
{
    /**
     * Sqlite database connection
     *
     * @var Zend_Db_Adapter_Pdo_Sqlite
     */
    protected $_db = null;

    /**
     * Database table authentication adapter
     *
     * @var Zend_Auth_Adapter_DbTable
     */
    protected $_adapter = null;

    /**
     * Set up test configuration
     *
     * @return void
     */
    public function setUp()
    {
        $this->_db = new Zend_Db_Adapter_Pdo_Sqlite(
            array('dbname' => TESTS_ZEND_AUTH_ADAPTER_DBTABLE_PDO_SQLITE_DATABASE)
            );

        $sqlCreate = 'CREATE TABLE [users] ( '
                   . '[id] INTEGER  NOT NULL PRIMARY KEY, '
                   . '[username] VARCHAR(50) NOT NULL, '
                   . '[password] VARCHAR(32) NULL, '
                   . '[real_name] VARCHAR(150) NULL)';
        $this->_db->query($sqlCreate);

        $sqlInsert = 'INSERT INTO users (username, password, real_name) '
                   . 'VALUES ("my_username", "my_password", "My Real Name")';
        $this->_db->query($sqlInsert);

        $this->_adapter = new Zend_Auth_Adapter_DbTable($this->_db, 'users', 'username', 'password');
    }

    /**
     * Ensures expected behavior for authentication success
     *
     * @return void
     */
    public function testAuthenticateSuccess()
    {
        $this->_adapter->setIdentity('my_username');
        $this->_adapter->setCredential('my_password');
        $result = $this->_adapter->authenticate();
        $this->assertTrue($result->isValid());
    }

    /**
     * Ensures expected behavior for authentication success
     *
     * @return void
     */
    public function testAuthenticateSuccessWithTreatment()
    {
        $this->_adapter = new Zend_Auth_Adapter_DbTable($this->_db, 'users', 'username', 'password', '?');
        $this->_adapter->setIdentity('my_username');
        $this->_adapter->setCredential('my_password');
        $result = $this->_adapter->authenticate();
        $this->assertTrue($result->isValid());
    }

    /**
     * Ensures expected behavior for for authentication failure
     * reason: Identity not found.
     *
     */
    public function testAuthenticateFailureIdentityNotFound()
    {
        $this->_adapter->setIdentity('non_existent_username');
        $this->_adapter->setCredential('my_password');

        try {
            $result = $this->_adapter->authenticate();
            $this->assertEquals($result->getCode(), Zend_Auth_Result::FAILURE_IDENTITY_NOT_FOUND);
        } catch (Zend_Auth_Exception $e) {
            $this->fail('Exception should have been thrown');
        }
    }

    /**
     * Ensures expected behavior for for authentication failure
     * reason: Identity not found.
     *
     */
    public function testAuthenticateFailureIdentityAmbigious()
    {
        $sql_insert = 'INSERT INTO users (username, password, real_name) VALUES ("my_username", "my_password", "My Real Name")';
        $this->_db->query($sql_insert);

        $this->_adapter->setIdentity('my_username');
        $this->_adapter->setCredential('my_password');

        try {
            $result = $this->_adapter->authenticate();
            $this->assertEquals($result->getCode(), Zend_Auth_Result::FAILURE_IDENTITY_AMBIGUOUS);
        } catch (Zend_Auth_Exception $e) {
            $this->fail('Exception should have been thrown');
        }
    }

    /**
     * Ensures expected behavior for authentication failure because of a bad password
     *
     * @return void
     */
    public function testAuthenticateFailureInvalidCredential()
    {
        $this->_adapter->setIdentity('my_username');
        $this->_adapter->setCredential('my_password_bad');
        $result = $this->_adapter->authenticate();
        $this->assertFalse($result->isValid());
    }

    /**
     * Ensures that getResultRowObject() works for successful authentication
     *
     * @return void
     */
    public function testGetResultRow()
    {
        $this->_adapter->setIdentity('my_username');
        $this->_adapter->setCredential('my_password');
        $result = $this->_adapter->authenticate();
        $resultRow = $this->_adapter->getResultRowObject();
        $this->assertEquals($resultRow->username, 'my_username');
    }

    /**
     * Ensure that ResultRowObject returns only what told to be included
     *
     */
    public function testGetSpecificResultRow()
    {
        $this->_adapter->setIdentity('my_username');
        $this->_adapter->setCredential('my_password');
        $result = $this->_adapter->authenticate();
        $resultRow = $this->_adapter->getResultRowObject(array('username', 'real_name'));
        $this->assertEquals(serialize($resultRow), 'O:8:"stdClass":2:{s:8:"username";s:11:"my_username";s:9:"real_name";s:12:"My Real Name";}');
    }

    /**
     * Ensure that ResultRowObject returns an object has specific omissions
     *
     */
    public function testGetOmittedResultRow()
    {
        $this->_adapter->setIdentity('my_username');
        $this->_adapter->setCredential('my_password');
        $result = $this->_adapter->authenticate();
        $resultRow = $this->_adapter->getResultRowObject(null, 'password');
        $this->assertEquals(serialize($resultRow), 'O:8:"stdClass":3:{s:2:"id";s:1:"1";s:8:"username";s:11:"my_username";s:9:"real_name";s:12:"My Real Name";}');
    }

    /**
     * Ensure that exceptions are caught
     *
     */
    public function testCatchExceptionNoTable()
    {
        $adapter = new Zend_Auth_Adapter_DbTable($this->_db);

        try {
            $result = $adapter->authenticate();
            $this->fail('Exception should have been thrown');
        } catch (Zend_Auth_Exception $e) {
            $this->assertEquals($e->getMessage(), 'A table must be supplied for the Zend_Auth_Adapter_DbTable authentication adapter.');
        }
    }

    /**
     * Ensure that exceptions are caught
     *
     */
    public function testCatchExceptionNoIdentityColumn()
    {
        $adapter = new Zend_Auth_Adapter_DbTable($this->_db, 'users');

        try {
            $result = $adapter->authenticate();
            $this->fail('Exception should have been thrown');
        } catch (Zend_Auth_Exception $e) {
            $this->assertEquals($e->getMessage(), 'An identity column must be supplied for the Zend_Auth_Adapter_DbTable authentication adapter.');
        }
    }

    /**
     * Ensure that exceptions are caught
     *
     */
    public function testCatchExceptionNoCredentialColumn()
    {
        $adapter = new Zend_Auth_Adapter_DbTable($this->_db, 'users', 'username');

        try {
            $result = $adapter->authenticate();
            $this->fail('Exception should have been thrown');
        } catch (Zend_Auth_Exception $e) {
            $this->assertEquals($e->getMessage(), 'A credential column must be supplied for the Zend_Auth_Adapter_DbTable authentication adapter.');
        }
    }

    /**
     * Ensure that exceptions are caught
     *
     */
    public function testCatchExceptionNoIdentity()
    {
        try {
            $result = $this->_adapter->authenticate();
            $this->fail('Exception should have been thrown');
        } catch (Zend_Auth_Exception $e) {
            $this->assertEquals($e->getMessage(), 'A value for the identity was not provided prior to authentication with Zend_Auth_Adapter_DbTable.');
        }
    }

    /**
     * Ensure that exceptions are caught
     *
     */
    public function testCatchExceptionNoCredential()
    {
        $this->_adapter->setIdentity('my_username');

        try {
            $result = $this->_adapter->authenticate();
            $this->fail('Exception should have been thrown');
        } catch (Zend_Auth_Exception $e) {
            $this->assertEquals($e->getMessage(), 'A credential value was not provided prior to authentication with Zend_Auth_Adapter_DbTable.');
        }
    }

    /**
     * Ensure that exceptions are caught
     *
     */
    public function testCatchExceptionBadSql()
    {
        $this->_adapter->setTableName('bad_table_name');
        $this->_adapter->setIdentity('value');
        $this->_adapter->setCredential('value');

        try {
            $result = $this->_adapter->authenticate();
            $this->fail('Exception should have been thrown');
        } catch (Zend_Auth_Exception $e) {
            $this->assertEquals($e->getMessage(), 'The supplied parameters to Zend_Auth_Adapter_DbTable failed to produce a valid sql statement, please check table and column names for validity.');
        }
    }



}


class Zend_Auth_Adapter_DbTable_BasicSqliteTest_Skip extends Zend_Auth_Adapter_DbTable_BasicSqliteTest
{
    public function setUp()
    {
        $this->markTestSkipped('Zend_Auth_Adapter_DbTable Sqlite tests are not enabled in TestConfiguration.php');
    }
}
