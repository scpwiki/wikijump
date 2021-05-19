<h1><a href="javascript:;" onclick="Wikijump.modules.AccountModule.utils.loadModule('am-profile')">{t}Your profile{/t}</a> / {t}Buddy icon{/t}</h1>

<p>
	{t}The buddy icon (also known as 'avatar') is a small image that represents you on this Wikijump installation.
	To be more precise - the buddy icon appears near users' screen names wherever possible as you have probably
	noticed.{/t}
</p>
{if isset($hasAvatar)}
<h2>{t}My current avatar{/t}:</h2>
<p style="text-align: center">
	<table style="margin: 0 auto;"><tr>
		<td><img src="{$avatarUri}" alt="" style="border: 1px solid #444;"/></td>
		<td>
			<input class="button" type="button" value="{t}delete{/t}?" onclick="Wikijump.modules.APAvatarModule.listeners.deleteAvatar(event)"/>
		</td>
	</tr></table>
</p>
<p>
	{t}Delete it if you wish. However due to caching issues you might need to
	hit "reload" in your browser to see the changes. It might also take max 1 hour for the
	change to be visible for others depending on their image caching.{/t}
</p>
{else}
<p>
	{t}Currently you have no buddy icon. If you want to upload one, please use the options below!{/t}
</p>

<ul id="avatar-choice1">
	<li><a href="javascript:;" onclick="$('avatar-choice1').style.display='none';$('file-upload-div').style.display='block'">{t}upload from your computer{/t}</a></li>
	<li><a href="javascript:;"  onclick="$('avatar-choice1').style.display='none';$('uri-upload-div').style.display='block'">{t}download from an existing web location{/t}</a></li>
</ul>

<div id="file-upload-div"  style="display: none">
	<form id="file-upload-form" enctype="multipart/form-data"
		action="/default--flow/Account__AvatarUploadTarget" target="_upload_iframe" method="POST"
		onsubmit="Wikijump.modules.APAvatarModule.listeners.startUpload(event)"
		>
		<input type="hidden" name="action" value="AccountProfileAction"/>
		<input type="hidden" name="event" value="uploadAvatar"/>

		<!-- MAX_FILE_SIZE must precede the file input field -->
	    <input type="hidden" name="MAX_FILE_SIZE" value="{$maxUpload}" />
	    <!-- Name of input element determines name in $_FILES array -->
	   	<table class="form">
	   		<tr>
	   			<td>
				    {t}Choose a file to upload{/t}:
				</td>
				<td>
				    <input name="userfile" type="file" size="30"/>
				</td>
			</tr>
		</table>

		<div class="buttons">
		   	<input class="button" type="button" value="{t}cancel{/t}" onclick="Wikijump.modules.APAvatarModule.listeners.reset(event)"/>
		   	<input class="button" type="submit" value="{t}upload file{/t}" />
		</div>
	</form>
</div>

<div id="uri-upload-div"  style="display: none">
	<form id="uri-upload-form">
		<table class="form">
	   		<tr>
	   			<td>
					{t}Enter image web address (URL){/t}:
				</td>
				<td>
					<input class="test" id="upload-uri" type="text" size="30" maxlength="80"/>
					<div class="sub">
						{t}start with <em>http://</em> or <em>ftp://</em>{/t}
					</div>
				</td>
			</tr>
		</table>
		<div class="buttons">
			<input class="button" type="button" value="{t}cancel{/t}" onclick="Wikijump.modules.APAvatarModule.listeners.reset(event)"/>
			<input class="button" type="button" value="{t}get it{/t}" onclick="Wikijump.modules.APAvatarModule.listeners.uploadUri(event)"/>
		</div>
	</form>
</div>

<iframe src="" name="_upload_iframe" id="_upload_iframe" style="display: none" ></iframe>

<div id="upload-wait" class="wait-progress" style="display: none"></div>

<div id="avatar-preview"  style="display: none">
	<h2>{t}Preview{/t}:</h2>
	<div style="text-align: center">
		<img src="" alt="{t}wrong image?{/t}" style="border: 1px solid #777;" id="avatar-preview-large"/>&nbsp;&nbsp;&nbsp;&nbsp;<img src="" alt="{t}wrong image?{/t}"  style="border: 1px solid #777;"  id="avatar-preview-small"/>
	</div>
	<p>
		{t}Above you shoud see the uploaded image. There should be an image
		scaled down (if was larger) to 100x100px format and 16x16px square format.{/t}
	</p>
	<p>
		{t}Note: Due to caching issues you might need to
		hit "reload" in your browser to see the changes. It might also take max 1 hour for the
		change to be visible for others depending on their image caching.{/t}
	</p>
	<p>
		{t}Do you want to use this image as your buddy icon?{/t}
		<div class="buttons">
			<input type="button" value="{t}no, never mind{/t}" onclick="Wikijump.modules.APAvatarModule.listeners.reset(event)"/>
			<input type="button" value="{t}use it!{/t}" onclick="Wikijump.modules.APAvatarModule.listeners.useIt(event)"/>

	</p>
</div>

<div id="avatar-success" style="display: none">
	<h2>{t}Success!!!{/t}</h2>
	{t}Your buddy icon has been successfully changed! However due to caching issues you might need to
	hit "reload" in your browser to see the changes. It might also take max 1 hour for the
	change to be visible for others depending on their image caching.{/t}
</div>
{/if}
