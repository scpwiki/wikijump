<?php
/**
 * Wikidot - free wiki collaboration software
 * Copyright (c) 2008, Wikidot Inc.
 * 
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * For more information about licensing visit:
 * http://www.wikidot.org/license
 * 
 * @category Ozone
 * @package Ozone_Email
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */
 
require_once PathManager::externalLibFile('phpmailer/class.phpmailer.php');

/**
 * A wrapper class around PHPMailer to conform to the coding standards.
 */
class PHPMailerWrap {
	private $phpMailer;
	
	public function __construct(){
		$this->phpMailer = new PHPMailer();
	}
	
	public function setAltBody($value){
		$this->phpMailer->AltBody = $value;	
	}
	
	public function getAltBody(){
		return $this->phpMailer->AltBody;	
	}
	
	public function setBody($value){
		$this->phpMailer->Body = $value;	
	}
	
	public function getBody(){
		return $this->phpMailer->Body;	
	}
	
	public function setCharSet($value){
		$this->phpMailer->CharSet = $value;	
	}
	
	public function getCharSet(){
		return $this->phpMailer->CharSet;	
	}
	
	public function setConfirmReadingTo($value){
		$this->phpMailer->ConfirmReadingTo = $value;	
	}
	
	public function getConfirmReadingTo(){
		return $this->phpMailer->ConfirmReadingTo;	
	}
	
	public function setContentType($value){
		$this->phpMailer->ContentType = $value;	
	}
	
	public function getContentType(){
		return $this->phpMailer->ContentType;	
	}	
	
	public function setEncoding($value){
		$this->phpMailer->Encoding = $value;	
	}
	
	public function getEncoding(){
		return $this->phpMailer->Encoding;	
	}	

	public function setErrorInfo($value){
		$this->phpMailer->ErrorInfo = $value;	
	}
	
	public function getErrorInfo(){
		return $this->phpMailer->ErrorInfo;	
	}		

	public function setFrom($value){
		$this->phpMailer->From = $value;	
	}
	
	public function getFrom(){
		return $this->phpMailer->From;	
	}
	
	public function setFromName($value){
		$this->phpMailer->FromName = $value;	
	}
	
	public function getFromName(){
		return $this->phpMailer->FromName;	
	}
	
	public function setHelo($value){
		$this->phpMailer->Helo = $value;	
	}
	
	public function getHelo(){
		return $this->phpMailer->Helo;	
	}
	
	public function setHost($value){
		$this->phpMailer->Host = $value;	
	}
	
	public function getHost(){
		return $this->phpMailer->Host;	
	}
	
	public function setHostname($value){
		$this->phpMailer->Hostname = $value;	
	}
	
	public function getHostname(){
		return $this->phpMailer->Hostname;	
	}
	
	public function setMailer($value){
		$this->phpMailer->Mailer = $value;	
	}
	
	public function getMailer(){
		return $this->phpMailer->Mailer;	
	}	
	
	public function setPassword($value){
		$this->phpMailer->Password = $value;	
	}
	
	public function getPassword(){
		return $this->phpMailer->Password;	
	}
	
	public function setPluginDir($value){
		$this->phpMailer->PluginDir = $value;	
	}
	
	public function getPluginDir(){
		return $this->phpMailer->PluginDir;	
	}
	
	public function setPort($value){
		$this->phpMailer->Port = $value;	
	}
	
	public function getPort(){
		return $this->phpMailer->Port;	
	}
	
	public function setPriority($value){
		$this->phpMailer->Priority = $value;	
	}
	
	public function getPriority(){
		return $this->phpMailer->Priority;	
	}
	
	public function setSender($value){
		$this->phpMailer->Sender = $value;	
	}
	
	public function getSender(){
		return $this->phpMailer->Sender;	
	}
	
	public function setSendmail(){
		$this->phpMailer->IsSendmail();	
	}

	public function setSMTPAuth($value){
		$this->phpMailer->SMTPAuth = $value;	
	}
	
	public function getSMTPAuth(){
		return $this->phpMailer->SMTPAuth;	
	}
	
	public function setSMTPDebug($value){
		$this->phpMailer->SMTPDebug = $value;	
	}
	
	public function getSMTPDebug(){
		return $this->phpMailer->SMTPDebug;	
	}
	
	public function setSMTPKeepAlive($value){
		$this->phpMailer->SMTPKeepAlive = $value;	
	}
	
	public function getSMTPKeepAlive(){
		return $this->phpMailer->SMTPKeepAlive;	
	}
	
	public function setSubject($value){
		$this->phpMailer->Subject = $value;	
	}
	
	public function getSubject(){
		return $this->phpMailer->Subject;	
	}
	
	public function setTimeout($value){
		$this->phpMailer->Timeout = $value;	
	}
	
	public function getTimeout(){
		return $this->phpMailer->Timeout;	
	}
	
	public function setUsername($value){
		$this->phpMailer->Username = $value;	
	}
	
	public function getUsername(){
		return $this->phpMailer->Username;	
	}
	
	public function setVersion($value){
		$this->phpMailer->Version = $value;	
	}
	
	public function getVersion(){
		return $this->phpMailer->Version;	
	}
	
	public function setWordWrap($value){
		$this->phpMailer->WordWrap = $value;	
	}
	
	public function getWordWrap(){
		return $this->phpMailer->WordWrap;	
	}
	
	public function addAddress($address, $name=""){
		$this->phpMailer->AddAddress($address, $name);	
	}
	
	public function addAttachment($path, $name = "", $encoding = "base64",$type = "application/octet-stream"){
		return $this->phpMailer->AddAttachmen($path, $name, $encoding ,$type);	
	}

	public function send(){
		return $this->phpMailer->Send();
	}
	
	public function setSMTP(){
		$this->phpMailer->IsSMTP();
	}
	
	public function setSMTPSecure($val){
		$this->phpMailer->SMTPSecure = $val;
	}
	
	public function setHtml($val){
		$this->phpMailer->IsHTML($val);
	}
}
