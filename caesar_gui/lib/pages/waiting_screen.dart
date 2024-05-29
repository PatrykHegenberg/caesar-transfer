import 'package:flutter/material.dart';
import '../messages/generated.dart';
import 'package:caesar_transfer/messages/ressource.pb.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:cross_file/cross_file.dart';
import 'package:shared_preferences/shared_preferences.dart';

const backColor = Color(0xFF32363E);
const highlightColor = Color(0xFF98C379);
const textColor = Color(0xFFABB2BF);

class WaitingScreen extends StatefulWidget {
  final List<XFile> files;
  final String transferName;

  const WaitingScreen(
      {Key? key, required this.transferName, required this.files})
      : super(key: key);

  @override
  _WaitingScreenState createState() => _WaitingScreenState();
}

class _WaitingScreenState extends State<WaitingScreen> {
  String appOrigin = '';
  @override
  void initState() {
    super.initState();
    loadSettings().then((_) => callStartSender(appOrigin));
  }

  Future<void> loadSettings() async {
    SharedPreferences prefs = await SharedPreferences.getInstance();
    appOrigin = prefs.getString('app_origin') ??
        'wss://caesar-transfer-iu.shuttleapp.rs'; // Laden Sie die app_origin
  }

  Future<void> callStartSender(String appOrigin) async {
    _sendFilenamestoRust(appOrigin);
  }

  void _sendFilenamestoRust(String appOrigin) {
    List<String> fileNames = widget.files.map((file) => file.path).toList();
    Files(relay: appOrigin, filenames: fileNames).sendSignalToRust();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: backColor,
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
              foregroundColor: highlightColor,
            ),
          ],
        ),
      ),
    );
  }
}
