<?php
chdir(dirname(__FILE__)); // unifies CLI/CGI cwd handling
require ('../php/setup.php');

// initialize things now

$logger = OzoneLogger::instance();
$loggerFileOutput = new OzoneLoggerFileOutput();
$loggerFileOutput->setLogFileName(WIKIJUMP_ROOT."/logs/jobs.log");
$logger->addLoggerOutput($loggerFileOutput);
$logger->setDebugLevel("debug");

$logger->debug("request processing started, logger initialized");

// initialize OZONE object too
Ozone::init();
$runData = new RunData();
$runData->init();
Ozone::setRunData($runData);

// Set the text domain as 'messages'
$gdomain = 'messages';
bindtextdomain($gdomain, WIKIJUMP_ROOT.'/locale');
textdomain($gdomain);


$jobName = $argv[1];

$classFile = WIKIJUMP_ROOT.'/php/jobs/'.$jobName.'.php';

require_once $classFile;

$job = new $jobName();

$job->run();
