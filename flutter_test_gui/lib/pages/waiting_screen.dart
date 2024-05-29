import 'package:flutter/material.dart';
// import 'package:flutter_test_gui/pages/send_screen.dart';
import 'package:flutter_test_gui/main.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:cross_file/cross_file.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:flutter_test_gui/src/rust/api/simple.dart';
// import 'package:flutter_test_gui/src/rust/frb_generated.dart';
import 'package:flutter_test_gui/consts/consts.dart';

class WaitingScreen extends StatefulWidget {
  final List<XFile> files;
  final String transferName;

  const WaitingScreen(
      {Key? key, required this.transferName, required this.files})
      : super(key: key);

  @override
  WaitingScreenState createState() => WaitingScreenState();
}

class WaitingScreenState extends State<WaitingScreen> {
  String appOrigin = '';
  @override
  void initState() {
    super.initState();
    loadSettings().then((_) => callStartSender(appOrigin));
  }

  Future<void> loadSettings() async {
    SharedPreferences prefs = await SharedPreferences.getInstance();
    appOrigin = prefs.getString('app_origin') ??
        'wss://caesar-transfer-iu.shuttleapp.rs';
  }

  Future<void> callStartSender(String appOrigin) async {
    _startTransfer(appOrigin);
  }

  Future<void> _startTransfer(String appOrigin) async {
    List<String> fileNames = widget.files.map((file) => file.path).toList();
    final outcome = await startRustSender(
        name: widget.transferName, relay: appOrigin, files: fileNames);
    Navigator.push(
        context,
        MaterialPageRoute(
            builder: (context) => MyHomePage(title: 'Caesar Transfer')));
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Constants.backColor,
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text(
              widget.transferName,
              style: const TextStyle(color: Colors.white, fontSize: 24),
            ),
            const SizedBox(height: 32),
            QrImageView(
              data: widget.transferName,
              version: QrVersions.auto,
              size: 200,
              foregroundColor: Constants.highlightColor,
            ),
          ],
        ),
      ),
    );
  }
}
