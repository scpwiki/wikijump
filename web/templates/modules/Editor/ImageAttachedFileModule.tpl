{if isset($files)}
	<p>
		Choose one of the files attached to this page and listed below:
	</p>

	<table style="margin: 0 auto">
		<tr>
			<td>
				<img id="wd-ed-imagewizard-byfile-preview" src="" alt=""/>
			</td>
			<td>
				<select  class="select" size="4" id="wd-ed-imagewizard-byfile-filename" onchange="Wikijump.Editor.imageWizard.attachementSelect(event)">
					{foreach from=$files item=file}
						<option value="{$file->getFilename()|escape}">{$file->getFilename()|escape}</option>
					{/foreach}
				</select>
			</td>
		</tr>
	</table>

	<table class="form">
		<tr>
			<td>
				Image size:
			</td>
			<td>
				<select  class="select" id="wd-ed-imagewizard-size">
					<option value="">original size</option>
					<option value="square">square - 75x75 pixels</option>
					<option value="thumbnail">thumbnail - 100 on longest side</option>
					<option value="small">small - 240 on longest side</option>
					<option value="medium">medium - 500 on longest side</option>
				</select>
			</td>
		</tr>
	</div>
{else}
	<p>
		Sorry, no images attached to this page found.
	</p>
{/if}
