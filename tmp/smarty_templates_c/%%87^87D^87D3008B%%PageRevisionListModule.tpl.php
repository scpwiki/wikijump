<?php /* Smarty version 2.6.7, created on 2009-01-04 19:43:53
         compiled from /var/www/wikidot/templates/modules/history/PageRevisionListModule.tpl */ ?>
<?php require_once(SMARTY_CORE_DIR . 'core.load_plugins.php');
smarty_core_load_plugins(array('plugins' => array(array('function', 'pager', '/var/www/wikidot/templates/modules/history/PageRevisionListModule.tpl', 1, false),array('function', 'printuser', '/var/www/wikidot/templates/modules/history/PageRevisionListModule.tpl', 63, false),array('block', 't', '/var/www/wikidot/templates/modules/history/PageRevisionListModule.tpl', 6, false),array('modifier', 'escape', '/var/www/wikidot/templates/modules/history/PageRevisionListModule.tpl', 65, false),)), $this); ?>
<?php echo smarty_function_pager(array('jsfunction' => 'updatePagedList','total' => $this->_tpl_vars['pagerData']['total_pages'],'known' => $this->_tpl_vars['pagerData']['known_pages'],'current' => $this->_tpl_vars['pagerData']['current_page']), $this);?>


<table class="page-history">
	<tr>
		<td>
			<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>rev.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
		</td>
		<td>
			&nbsp;
		</td>
		<td>
			<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>flags<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
		</td>
		<td>
			<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>actions<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
		</td>
		<td>
			<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>by<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
		</td>
		<td>
			<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>date<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
		</td>
		<td>
			<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>comments<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
		</td>
	</tr>
	<?php $this->assign('count', 0); ?>
	<?php if (count($_from = (array)$this->_tpl_vars['revisions'])):
    foreach ($_from as $this->_tpl_vars['pr']):
?>
	
	<tr id="revision-row-<?php echo $this->_tpl_vars['pr']->getRevisionId(); ?>
">
		<td><?php echo $this->_tpl_vars['pr']->getRevisionNumber(); ?>
.</td>
		<td style="width: 5em" >
			<input id="<?php echo $this->_tpl_vars['pr']->getRevisionId(); ?>
" type="radio" name="from" value="<?php echo $this->_tpl_vars['pr']->getRevisionId(); ?>
" <?php if ($this->_tpl_vars['count'] == 1): ?>checked="checked" <?php $this->assign('count', 2);  endif; ?> />
			<input id="<?php echo $this->_tpl_vars['pr']->getRevisionId(); ?>
" type="radio" name="to" value="<?php echo $this->_tpl_vars['pr']->getRevisionId(); ?>
" <?php if ($this->_tpl_vars['count'] == 0): ?>checked="checked" <?php $this->assign('count', 1);  endif; ?> />
		</td>
		<td>
			<?php if ($this->_tpl_vars['pr']->getFlagNew()): ?>
		 		<span class="spantip" title="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>new page created<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>">N</span>
		 	<?php endif; ?>
		 	<?php if ($this->_tpl_vars['pr']->getFlagText()): ?>
		 		<span class="spantip" title="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>content source text changed<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>">S</span>
		 	<?php endif; ?>
		 	<?php if ($this->_tpl_vars['pr']->getFlagTitle()): ?>
		 		<span class="spantip" title="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>title changed<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>">T</span>
		 	<?php endif; ?>
		 	<?php if ($this->_tpl_vars['pr']->getFlagRename()): ?>
		 		<span class="spantip" title="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>page renamed/moved<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>">R</span>
		 	<?php endif; ?>  
		 	<?php if ($this->_tpl_vars['pr']->getFlagFile()): ?>
		 		<span class="spantip" title="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>file/attachment action<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>">F</span>
		 	<?php endif; ?>  
		 	<?php if ($this->_tpl_vars['pr']->getFlagMeta()): ?>
		 		<span class="spantip" title="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>meta data changed<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>">M</span>
		 	<?php endif; ?> 
		</td>
		<td style="width: 5em" class="optionstd">
			 <a title="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>view page revision<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>" href="javascript:;" onclick="showVersion(<?php echo $this->_tpl_vars['pr']->getRevisionId(); ?>
)">V</a>
			 <a title="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>view source of the revision<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>" href="javascript:;" onclick="showSource(<?php echo $this->_tpl_vars['pr']->getRevisionId(); ?>
)">S</a>
			 <?php if (( $this->_tpl_vars['pr']->getFlagNew() || $this->_tpl_vars['pr']->getFlagText() || $this->_tpl_vars['pr']->getFlagTitle() ) && $this->_tpl_vars['page']->getRevisionId() != $this->_tpl_vars['pr']->getRevisionId()): ?> 			 	<a title="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>revert to revision<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>" href="javascript:;" onclick="WIKIDOT.modules.PageHistoryModule.listeners.revert(event,<?php echo $this->_tpl_vars['pr']->getRevisionId(); ?>
)">R</a>
			 <?php endif; ?>
		</td>
		<td style="width: 15em"><?php echo smarty_function_printuser(array('user' => $this->_tpl_vars['pr']->getUserOrString(),'image' => 'true'), $this);?>
</td>
		<td style="padding: 0 0.5em; width: 7em;"><span class="odate"><?php echo $this->_tpl_vars['pr']->getDateLastEdited()->getTimestamp(); ?>
|%e %b %Y|agohover</span></td>
		<td style="font-size: 90%"><?php echo ((is_array($_tmp=$this->_tpl_vars['pr']->getComments())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
</td>
	</tr>
	<?php endforeach; endif; unset($_from); ?>
</table>