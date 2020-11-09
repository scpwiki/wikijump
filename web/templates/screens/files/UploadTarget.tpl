{$page->setLayout("Plain")}
{$status} {$message}
<script type="text/javascript">
//alert("asdasdasd");
window.parent.Wikijump.modules.PageUploadModule.listeners.fileUploaded("{$status}", "{$message}");
//window.parent.test("from upload targer");
</script>
