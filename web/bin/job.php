<?php

use Ozone\Framework\Ozone;
use Ozone\Framework\OzoneLogger;
use Ozone\Framework\OzoneLoggerFileOutput;
use Ozone\Framework\RunData;

chdir(dirname(__FILE__)); // unifies CLI/CGI cwd handling
require ('../php/setup.php');

// initialize things now

$logger = OzoneLogger::instance();
$loggerFileOutput = new OzoneLoggerFileOutput();
$loggerFileOutput->setLogFileName(WIKIJUMP_ROOT."/logs/jobs.log");
$logger->addLoggerOutput($loggerFileOutput);

$logger->debug("request processing started, logger initialized");

// initialize OZONE object too
Ozone::init();
$runData = new RunData();
$runData->init();
Ozone::setRunData($runData);

$jobName = $argv[1];

$classFile = WIKIJUMP_ROOT.'/php/Jobs/'.$jobName.'.php';

require_once $classFile;

$job = new $jobName();

$job->run();
