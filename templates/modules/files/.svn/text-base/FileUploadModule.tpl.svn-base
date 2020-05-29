<h2>{t}File upload{/t}</h2>
<p>
	{t}Current site storage size{/t}: {$totalSiteSize} {t}used of total{/t} {$totalSiteAllowedSize}, {$availableSiteSize} {t}still free{/t}
</p>
<p>
	{t}Max upload size{/t}: {$maxUploadString} 
</p>


<form id="file-upload-form" enctype="multipart/form-data" 
	action="/default--flow/files__UploadTarget" target="_upload_iframe" method="POST">
	<div style="text-align: center">
	
		<input type="hidden" name="action" value="FileAction"/>
		<input type="hidden" name="event" value="uploadFile"/>
		<input type="hidden" name="page_id" id="file-upload-form-page-id" value=""/>
		<!-- MAX_FILE_SIZE must precede the file input field -->
	    <input type="hidden" name="MAX_FILE_SIZE" value="{$maxUpload}" />
	    <!-- Name of input element determines name in $_FILES array -->
	    <table class="form">
		    	<tr>
		    		<td>{t}File to upload:{/t}</td>
		    		<td><input class="text" id="upload-userfile" name="userfile" type="file" size="30"/></td>
		    	</tr>
		    	<tr>
	    			<td>{t}Destination file name:{/t}</td>
	    			<td>
	    				<input class="text" id="upload-dfilename" name="dfilename" type="text" size="30"/>
	    				<div class="sub">
	    					{t}Leave blank to keep the original name.{/t}
	    				</div>
	    			</td>
	    		</tr>
	    		<tr>
	    			<td>{t}File comments:{/t}</td>
	    			<td>
	    				<textarea name="comments" id="file-comments" cols="30" rows="2"></textarea>
		    			<div class="sub">
		    				{t escape=no}Max 100 characters (<span id="file-comments-charleft"></span> left){/t}
					</div>
				</td>
	    		</tr>
	    	</table>
	    <div class="buttons">
		    <input type="button" value="{t}cancel{/t}" onclick="WIKIDOT.modules.PageUploadModule.listeners.uploadCancel(event)"/>
		    <input type="button" value="{t}upload file!{/t}" onclick="WIKIDOT.modules.PageUploadModule.listeners.checkFileExists(event)"/>
		</div>
    </div>
</form>

<iframe src="/common--misc/blank.html" name="_upload_iframe" id="_upload_iframe" style="display: none;"></iframe>